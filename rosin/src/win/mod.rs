// TODO:
//  - Queueing & Blocking on thread
//      - Make a noop abstraction layer because the functions **may** be thread safe
//      - I need to be able to remember the queue per thread
//      - I would need probably some use of static?
//      - Have an inbetween abstraction layer between what `WindowHandle` does and what `RosinView` does?
//  - Rendering & Drawing
//  - (eventually) Try to remove as much dependency on unsafe as possible

pub mod app;
pub mod handle;

mod proc_fn;
mod view;
