
// TODO: The goal is for this to grow and hold generic functions
// and provide a single trait for any platform implementation...
// as it stands I don't understand how to reference and make use
// of a trait not defined in the same file... wat

trait Platform {
  fn exit_requested(&self) -> bool;
  fn process_events(&self);
  fn swap(&self);
}
