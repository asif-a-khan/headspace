/// View permission scoping helper.
///
/// Returns a SQL WHERE clause fragment that filters entities by user_id
/// based on the current user's view_permission setting.
pub fn view_permission_filter(user_id: i64, view_permission: &str) -> String {
    match view_permission {
        "global" => String::new(),
        "group" => format!(
            " AND t.user_id IN (
                SELECT ug.user_id FROM user_groups ug
                WHERE ug.group_id IN (SELECT group_id FROM user_groups WHERE user_id = {user_id})
            )"
        ),
        // "individual" or anything else
        _ => format!(" AND t.user_id = {user_id}"),
    }
}
