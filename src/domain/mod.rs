pub mod new_subscriber;
pub mod subscriber_email;
pub mod subscriber_name;

// Useful to do this while refactoring to prevent breaking imports
pub use new_subscriber::NewSubscriber;
pub use subscriber_email::SubscriberEmail;
pub use subscriber_name::SubscriberName;
