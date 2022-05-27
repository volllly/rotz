pub mod clone;
pub mod install;
pub mod link;
pub use clone::Clone;
pub use install::Install;
pub use link::Link;

pub trait Command {
  type Args;
  type Result;

  fn execute(&self, args: Self::Args) -> Self::Result;
}
