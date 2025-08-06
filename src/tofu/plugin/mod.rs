pub mod tfplugin6 {
    pub struct Provider {}
    impl Provider {
        pub fn new() -> Self { Provider {} }
        pub fn configure(&self) -> Result<(), String> { Ok(()) }
    }
}
