use hyper::Method;

use gotham::handler::NewHandler;
use gotham::middleware::pipeline::new_pipeline;
use gotham::middleware::session::NewSessionMiddleware;
use gotham::router::Router;
use gotham::router::route::{Extractors, Route, RouteImpl, Delegation};
use gotham::router::route::dispatch::{new_pipeline_set, finalize_pipeline_set, PipelineSet,
                                      PipelineHandleChain, DispatcherImpl};
use gotham::router::route::matcher::MethodOnlyRouteMatcher;
use gotham::router::request::path::NoopPathExtractor;
use gotham::router::request::query_string::NoopQueryStringExtractor;
use gotham::router::response::finalizer::ResponseFinalizerBuilder;
use gotham::router::tree::TreeBuilder;
use gotham::router::tree::node::{NodeBuilder, SegmentType};

use controllers::welcome;
use controllers::todo;
use controllers::challenge::{self, ChallengeRequestPath, ChallengeQueryString};
use session::Session;

// Please do a lot of squinting when considering Router setup in Gotham 0.1.
//
// In future Gotham releases we'll be focusing on streamlining this but if you look
// carefully you'll see this actually isn't to far off what you might
// already find in Rails/Phoenix/Django just type safe and Rust-y.

fn static_route<NH, P, C>(methods: Vec<Method>,
                          new_handler: NH,
                          active_pipelines: C,
                          ps: PipelineSet<P>)
                          -> Box<Route + Send + Sync>
    where NH: NewHandler + 'static,
          C: PipelineHandleChain<P> + Send + Sync + 'static,
          P: Send + Sync + 'static
{
    // Requests must have used the specified method(s) in order for this Route to match.
    //
    // You could define your on RouteMatcher of course.. perhaps you'd like to only match on
    // requests that are made using the GET method and send a User-Agent header for a particular
    // version of browser you'd like to make fun of....
    let matcher = MethodOnlyRouteMatcher::new(methods);

    // For Requests that match this Route we'll dispatch them to new_handler via the pipelines
    // defined in active_pipelines.
    //
    // n.b. We also specify the set of all known pipelines in the application so the dispatcher can
    // resolve the pipeline references provided in active_pipelines. For this application that is
    // only the global pipeline.
    let dispatcher = DispatcherImpl::new(new_handler, active_pipelines, ps);
    let extractors: Extractors<NoopPathExtractor, NoopQueryStringExtractor> = Extractors::new();
    let route = RouteImpl::new(matcher,
                               Box::new(dispatcher),
                               extractors,
                               Delegation::Internal);
    Box::new(route)
}

fn challenge_route<NH, P, C>(methods: Vec<Method>,
                             new_handler: NH,
                             active_pipelines: C,
                             pipeline_set: PipelineSet<P>)
                             -> Box<Route + Send + Sync>
    where NH: NewHandler + 'static,
          C: PipelineHandleChain<P> + Send + Sync + 'static,
          P: Send + Sync + 'static
{
    let matcher = MethodOnlyRouteMatcher::new(methods);
    let dispatcher = DispatcherImpl::new(new_handler, active_pipelines, pipeline_set);

    // Note the Route isn't simply not caring about the Request path and Query string. It will
    // extract data from both, in a type safe way and safely deposit it into a instance of the
    // structs shown below, ready for use by Middleware and Handlers (Usually just your handler,
    // which is a function in your controller).
    let extractors: Extractors<ChallengeRequestPath, ChallengeQueryString> = Extractors::new();
    let route = RouteImpl::new(matcher,
                               Box::new(dispatcher),
                               extractors,
                               Delegation::Internal);
    Box::new(route)
}

/// Creates a Router that represents the following tree:
///
/// /            (Routeable, GET and HEAD methods supported)
/// - /todo      (Routable, GET, HEAD and POST methods supported)
///   - /reset   (Routable, POST method supported)
/// - /challenge     (Not Routable)
///   - /name    (Dynamic matching, GET, HEAD methods supported)
///
pub fn router() -> Router {
    // Start to build the Tree structure which our Router will rely on to match and dispatch
    // Requests entering our application.
    let mut tree_builder = TreeBuilder::new();

    // There is a single PipelineSet in use for this Router, which we refer to as global.
    // It utilises a single `Middleware` that helps the application maintain data between Requests
    // by using an in memory backend.
    //
    // Pipelines are very powerful and can be nested at different levels in your application.
    //
    // You can also assign multiple Middleware instances to a Pipeline each will be evaluated in
    // order of definition for each Request entering the system.
    let ps_builder = new_pipeline_set();
    let (ps_builder, global) = ps_builder
        .add(new_pipeline()
                 .add(NewSessionMiddleware::default()
                          .insecure()
                          .with_session_type::<Session>())
                 .build());
    let ps = finalize_pipeline_set(ps_builder);

    // Add a Route directly to the root of our `Tree` so that `Requests` for `/` are handled by
    // the `welcome` controller. Each function within the `welcome` controller represents a complete
    // `Handler` in Gotham parlance.
    tree_builder.add_route(static_route(vec![Method::Get, Method::Head], // Use this Route for Get and Head Requests
                                        || Ok(welcome::index),
                                        (global, ()), // This signifies that the active Pipelines for this route consist only of the global pipeline
                                        ps.clone())); // All the pipelines we've created for this Router

    // Create a Node to represent the Request path /todo
    let mut todo = NodeBuilder::new("todo", SegmentType::Static);

    // Add `Routes` to the todo node of our `Tree` so that `Requests` for `/todo` are handled by
    // the `todo` controller. Each function within the `todo` controller represents a complete
    // `Handler` in Gotham parlance.
    todo.add_route(static_route(vec![Method::Get, Method::Head], // Use this Route for Get and Head Requests
                                        || Ok(todo::index),
                                        (global, ()), // This signifies that the active Pipelines for this route consist only of the global pipeline
                                        ps.clone())); // All the pipelines we've created for this Router

    todo.add_route(static_route(vec![Method::Post], // Use this Route for Post Requests
                                || Ok(todo::add),
                                (global, ()),
                                ps.clone()));

    // Create a Node to represent the Request path /reset and add a route to handle Post requests
    let mut reset = NodeBuilder::new("reset", SegmentType::Static);
    reset.add_route(static_route(vec![Method::Post],
                                 || Ok(todo::reset),
                                 (global, ()),
                                 ps.clone()));

    // Add the reset node to the todo node
    todo.add_child(reset);

    // Add the todo node to the tree to complete this path
    tree_builder.add_child(todo);

    // Create a Node to represent the Request path /challenge that can't handle requests directly
    let mut challenge = NodeBuilder::new("challenge", SegmentType::Static);

    // Create a Node that matches any segment value it sees and allows the handler to process it
    // through (type safe!!) Request path extraction
    let mut name = NodeBuilder::new("name", SegmentType::Dynamic);
    name.add_route(challenge_route(vec![Method::Get, Method::Head],
                                   || Ok(challenge::index),
                                   (global, ()),
                                   ps.clone()));

    // Add the name node to the challenge node
    challenge.add_child(name);

    // Add the challenge node to the tree to complete this path
    tree_builder.add_child(challenge);

    // We've setup all our Pipelines, Middleware, Nodes and Routes.
    //
    // IT'S GO TIME.
    //   - Izzy Mandelbaum
    //
    // Well... almost. First we need to finalize our Tree, meaning we can no longer make any changes
    // to it or any of the routing information it holds.
    let tree = tree_builder.finalize();

    // Oh I nearly forgot Response finalizers.
    //
    // We actually have no need for Response finalizers in this application.
    //
    // However they are pretty cool, check them out sometime when you're bored.
    let response_finalizer_builder = ResponseFinalizerBuilder::new();
    let response_finalizer = response_finalizer_builder.finalize();

    // NOW IT`S GO TIME
    Router::new(tree, response_finalizer)
}
