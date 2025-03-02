wit_bindgen::generate!({
    path: "wit",
    world: "myworld",
});


struct MyHost;

impl Guest for MyHost {
    fn change_user(user1: UserData) -> UserData {
        return UserData {
            first_name: String::from("Larry"),
            last_name: String::from("Page"),
            age: 67,
            grades: vec![15, 10, 20],
        }
    }

    fn get_name(user1: UserData) -> String {
        println!("Hello World");
        return format!("{} {}", user1.first_name, user1.last_name);
    }
}

export!(MyHost);