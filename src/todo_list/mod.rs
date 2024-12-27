use std::io::{self, Write};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, FixedOffset};
use bcrypt::{hash, verify, DEFAULT_COST};

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    id: usize,
    description: String,
    completed: bool,
    created_at: DateTime<FixedOffset> ,
    user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    username: String,
    password_hash: String,
}

#[derive(Debug)]
struct TodoApp {
    tasks: HashMap<usize, Task>,
    users: HashMap<String, User>,
    current_user: Option<String>,
    next_id: usize,
}

impl TodoApp {
    fn new() -> Self {
        Self {
            tasks: Self::load_tasks().unwrap_or_default(),
            users: Self::load_users().unwrap_or_default(),
            current_user: None,
            next_id: 1,
        }
    }

    fn save_tasks(&self) -> io::Result<()> {
        let json = serde_json::to_string(&self.tasks)?;
        fs::write("tasks.json", json)?;
        Ok(())
    }

    fn load_tasks() -> io::Result<HashMap<usize, Task>> {
        if Path::new("tasks.json").exists() {
            let data = fs::read_to_string("tasks.json")?;
            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(HashMap::new())
        }
    }

    fn save_users(&self) -> io::Result<()> {
        let json = serde_json::to_string(&self.users)?;
        fs::write("users.json", json)?;
        Ok(())
    }

    fn load_users() -> io::Result<HashMap<String, User>> {
        if Path::new("users.json").exists() {
            let data = fs::read_to_string("users.json")?;
            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(HashMap::new())
        }
    }

    fn register(&mut self, username: &str, password: &str) -> bool {
        if self.users.contains_key(username) {
            println!("User already exists!");
            return false;
        }

        let password_hash = hash(password.as_bytes(), DEFAULT_COST).unwrap();
        self.users.insert(username.to_string(), User {
            username: username.to_string(),
            password_hash,
        });
        self.save_users().unwrap();
        true
    }

    fn login(&mut self, username: &str, password: &str) -> bool {
        if let Some(user) = self.users.get(username) {
            if verify(password.as_bytes(), &user.password_hash).unwrap() {
                self.current_user = Some(username.to_string());
                return true;
            }
        }
        false
    }

    fn add_task(&mut self, description: String) -> bool {
        if let Some(user_id) = &self.current_user {
            let offset = FixedOffset::east_opt(2 * 3600).unwrap();
            let now = Utc::now().with_timezone(&offset);

            let task = Task {
                id: self.next_id,
                description,
                completed: false,
                created_at: now,
                user_id: user_id.clone(),
            };
            self.tasks.insert(self.next_id, task);
            self.next_id += 1;
            self.save_tasks().unwrap();
            true
        } else {
            false
        }
    }

    fn edit_task(&mut self, id: usize, new_description: String) -> bool {
        if let Some(task) = self.tasks.get_mut(&id) {
            if let Some(user_id) = &self.current_user {
                if task.user_id == *user_id {
                    task.description = new_description;
                    self.save_tasks().unwrap();
                    return true;
                }
            }
        }
        false
    }

    fn delete_task(&mut self, id: usize) -> bool {
        if let Some(task) = self.tasks.get(&id) {
            if let Some(user_id) = &self.current_user {
                if task.user_id == *user_id {
                    self.tasks.remove(&id);
                    self.save_tasks().unwrap();
                    return true;
                }
            }
        }
        false
    }

    fn complete_task(&mut self, id: usize) -> bool {
        if let Some(task) = self.tasks.get_mut(&id) {
            if let Some(user_id) = &self.current_user {
                if task.user_id == *user_id {
                    task.completed = true;
                    self.save_tasks().unwrap();
                    return true;
                }
            }
        }
        false
    }

    fn list_tasks(&self) {
        if let Some(user_id) = &self.current_user {
            let mut user_tasks: Vec<_> = self.tasks.values()
                .filter(|task| task.user_id == *user_id)
                .collect();

            user_tasks.sort_by_key(|task| task.id);

            if user_tasks.is_empty() {
                println!("No tasks found!");
                return;
            }

            println!("\nYour TODO List:");
            println!("---------------");
            for task in user_tasks {
                let status = if task.completed { "✓" } else { " " };
                println!("{}. [{}] {} (Created: {})",
                         task.id,
                         status,
                         task.description,
                         task.created_at.format("%Y-%m-%d %H:%M")
                );
            }
            println!("---------------\n");
        } else {
            println!("Please log in first!");
        }
    }
}

pub fn run_todo() {
    let mut app = TodoApp::new();
    println!("Welcome to Todo List Manager!");

    loop {
        if app.current_user.is_none() {
            print!("Not logged in. Commands: register, login, exit\n> ");
        } else {
            print!("Logged in as {}. Commands: add, list, edit, delete, complete, logout, exit\n> ",
                   app.current_user.as_ref().unwrap());
        }

        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let command = parts[0].to_lowercase();

        match command.as_str() {
            "register" if app.current_user.is_none() => {
                print!("Enter username: ");
                io::stdout().flush().unwrap();
                let mut username = String::new();
                io::stdin().read_line(&mut username).unwrap();

                print!("Enter password: ");
                io::stdout().flush().unwrap();
                let mut password = String::new();
                io::stdin().read_line(&mut password).unwrap();

                if app.register(username.trim(), password.trim()) {
                    println!("Registration successful!");
                }
            },

            "login" if app.current_user.is_none() => {
                print!("Enter username: ");
                io::stdout().flush().unwrap();
                let mut username = String::new();
                io::stdin().read_line(&mut username).unwrap();

                print!("Enter password: ");
                io::stdout().flush().unwrap();
                let mut password = String::new();
                io::stdin().read_line(&mut password).unwrap();

                if app.login(username.trim(), password.trim()) {
                    println!("Login successful!");
                } else {
                    println!("Invalid username or password!");
                }
            },

            "logout" if app.current_user.is_some() => {
                app.current_user = None;
                println!("Logged out successfully!");
            },

            "add" if app.current_user.is_some() => {
                if parts.len() < 2 {
                    println!("Please provide a task description");
                    continue;
                }
                if app.add_task(parts[1].to_string()) {
                    println!("Task added successfully!");
                }
            },

            "edit" if app.current_user.is_some() => {
                print!("Enter task ID: ");
                io::stdout().flush().unwrap();
                let mut id = String::new();
                io::stdin().read_line(&mut id).unwrap();

                print!("Enter new description: ");
                io::stdout().flush().unwrap();
                let mut description = String::new();
                io::stdin().read_line(&mut description).unwrap();

                if let Ok(id) = id.trim().parse() {
                    if app.edit_task(id, description.trim().to_string()) {
                        println!("Task updated successfully!");
                    } else {
                        println!("Failed to update task!");
                    }
                }
            },

            "delete" if app.current_user.is_some() => {
                if parts.len() < 2 {
                    println!("Please provide a task ID");
                    continue;
                }
                if let Ok(id) = parts[1].parse() {
                    if app.delete_task(id) {
                        println!("Task deleted successfully!");
                    } else {
                        println!("Failed to delete task!");
                    }
                }
            },

            "complete" if app.current_user.is_some() => {
                if parts.len() < 2 {
                    println!("Please provide a task ID");
                    continue;
                }
                if let Ok(id) = parts[1].parse() {
                    if app.complete_task(id) {
                        println!("Task marked as completed!");
                    } else {
                        println!("Failed to complete task!");
                    }
                }
            },

            "list" if app.current_user.is_some() => {
                app.list_tasks();
            },

            "exit" => {
                println!("Goodbye!");
                break;
            },

            _ => println!("Unknown command or not logged in."),
        }
    }
}