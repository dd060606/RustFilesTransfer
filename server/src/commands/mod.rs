pub trait Command {
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn aliases(&self) -> Vec<String>;
    fn execute(&self);
}

pub struct CommandRegistry<'a> {
    commands: Vec<Box<dyn Command>>,
}

impl<'a> CommandRegistry<'a> {
    pub fn new() -> Self {
        let mut registry = Self {
            commands: Vec::new(),
        };
        // Register commands here
        //registry.register(Box::new(help::HelpCommand));
        //registry.register(Box::new(exit::ExitCommand));
        //registry.register(Box::new(list::ListCommand));

        registry
    }



    pub fn execute(&self, command_name: String) {
        if let Some(command) = self.commands.get(command_name) {
          //  command.execute(args);
        } else {
            println!("Unknown command: {}", command_name);
        }
    }
}