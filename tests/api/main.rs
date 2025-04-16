mod health_check;
mod helpers;
mod login;
mod newsletters;
mod subscriptions;
mod subscriptions_confirm;

// This pattern causes our integration tests to only be one executable
// with sub modules, instead of one crate per file, each duplicating the
// helper module.
