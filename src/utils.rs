use serenity::all::User;

pub fn user_to_name(user: &User) -> &str {
    user.global_name
        .as_ref()
        .map(|it| it.as_str())
        .unwrap_or(user.name.as_str())
}
