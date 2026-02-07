//! ACL permission tree definition.
//!
//! Defines the super admin permission tree. Each node has a key (dot notation),
//! display name, and optional children. The tree is flattened for template rendering.

use serde::Serialize;

/// A node in the ACL permission tree.
pub struct AclItem {
    pub key: &'static str,
    pub name: &'static str,
    pub children: &'static [AclItem],
}

/// Flattened ACL node for template/frontend rendering.
#[derive(Debug, Clone, Serialize)]
pub struct FlatAclNode {
    pub key: String,
    pub name: String,
    pub depth: usize,
}

/// Super admin ACL tree.
///
/// Permissions:
/// - tenants (create, edit, delete)
/// - settings.agents (create, edit, delete)
/// - settings.roles (create, edit, delete)
pub static SUPER_ADMIN_ACL: &[AclItem] = &[
    AclItem {
        key: "tenants",
        name: "Tenants",
        children: &[
            AclItem {
                key: "tenants.create",
                name: "Create",
                children: &[],
            },
            AclItem {
                key: "tenants.edit",
                name: "Edit",
                children: &[],
            },
            AclItem {
                key: "tenants.delete",
                name: "Delete",
                children: &[],
            },
        ],
    },
    AclItem {
        key: "settings",
        name: "Settings",
        children: &[
            AclItem {
                key: "settings.agents",
                name: "Agents",
                children: &[
                    AclItem {
                        key: "settings.agents.create",
                        name: "Create",
                        children: &[],
                    },
                    AclItem {
                        key: "settings.agents.edit",
                        name: "Edit",
                        children: &[],
                    },
                    AclItem {
                        key: "settings.agents.delete",
                        name: "Delete",
                        children: &[],
                    },
                ],
            },
            AclItem {
                key: "settings.roles",
                name: "Roles",
                children: &[
                    AclItem {
                        key: "settings.roles.create",
                        name: "Create",
                        children: &[],
                    },
                    AclItem {
                        key: "settings.roles.edit",
                        name: "Edit",
                        children: &[],
                    },
                    AclItem {
                        key: "settings.roles.delete",
                        name: "Delete",
                        children: &[],
                    },
                ],
            },
        ],
    },
];

/// Tenant admin ACL tree.
///
/// Permissions for CRM features:
/// - dashboard
/// - leads (create, edit, delete)
/// - quotes (create, edit, delete)
/// - mail
/// - activities (create, edit, delete)
/// - contacts.persons (create, edit, delete)
/// - contacts.organizations (create, edit, delete)
/// - products (create, edit, delete)
/// - settings.users (create, edit, delete)
/// - settings.roles (create, edit, delete)
pub static TENANT_ADMIN_ACL: &[AclItem] = &[
    AclItem {
        key: "dashboard",
        name: "Dashboard",
        children: &[],
    },
    AclItem {
        key: "leads",
        name: "Leads",
        children: &[
            AclItem { key: "leads.create", name: "Create", children: &[] },
            AclItem { key: "leads.edit", name: "Edit", children: &[] },
            AclItem { key: "leads.delete", name: "Delete", children: &[] },
        ],
    },
    AclItem {
        key: "quotes",
        name: "Quotes",
        children: &[
            AclItem { key: "quotes.create", name: "Create", children: &[] },
            AclItem { key: "quotes.edit", name: "Edit", children: &[] },
            AclItem { key: "quotes.delete", name: "Delete", children: &[] },
        ],
    },
    AclItem {
        key: "mail",
        name: "Mail",
        children: &[],
    },
    AclItem {
        key: "activities",
        name: "Activities",
        children: &[
            AclItem { key: "activities.create", name: "Create", children: &[] },
            AclItem { key: "activities.edit", name: "Edit", children: &[] },
            AclItem { key: "activities.delete", name: "Delete", children: &[] },
        ],
    },
    AclItem {
        key: "contacts",
        name: "Contacts",
        children: &[
            AclItem {
                key: "contacts.persons",
                name: "Persons",
                children: &[
                    AclItem { key: "contacts.persons.create", name: "Create", children: &[] },
                    AclItem { key: "contacts.persons.edit", name: "Edit", children: &[] },
                    AclItem { key: "contacts.persons.delete", name: "Delete", children: &[] },
                ],
            },
            AclItem {
                key: "contacts.organizations",
                name: "Organizations",
                children: &[
                    AclItem { key: "contacts.organizations.create", name: "Create", children: &[] },
                    AclItem { key: "contacts.organizations.edit", name: "Edit", children: &[] },
                    AclItem { key: "contacts.organizations.delete", name: "Delete", children: &[] },
                ],
            },
        ],
    },
    AclItem {
        key: "products",
        name: "Products",
        children: &[
            AclItem { key: "products.create", name: "Create", children: &[] },
            AclItem { key: "products.edit", name: "Edit", children: &[] },
            AclItem { key: "products.delete", name: "Delete", children: &[] },
        ],
    },
    AclItem {
        key: "settings",
        name: "Settings",
        children: &[
            AclItem {
                key: "settings.users",
                name: "Users",
                children: &[
                    AclItem { key: "settings.users.create", name: "Create", children: &[] },
                    AclItem { key: "settings.users.edit", name: "Edit", children: &[] },
                    AclItem { key: "settings.users.delete", name: "Delete", children: &[] },
                ],
            },
            AclItem {
                key: "settings.roles",
                name: "Roles",
                children: &[
                    AclItem { key: "settings.roles.create", name: "Create", children: &[] },
                    AclItem { key: "settings.roles.edit", name: "Edit", children: &[] },
                    AclItem { key: "settings.roles.delete", name: "Delete", children: &[] },
                ],
            },
        ],
    },
];

/// Flatten the ACL tree into a list of nodes with depth info.
pub fn flatten_acl(items: &[AclItem]) -> Vec<FlatAclNode> {
    let mut result = Vec::new();
    flatten_recursive(items, 0, &mut result);
    result
}

fn flatten_recursive(items: &[AclItem], depth: usize, result: &mut Vec<FlatAclNode>) {
    for item in items {
        result.push(FlatAclNode {
            key: item.key.to_string(),
            name: item.name.to_string(),
            depth,
        });
        flatten_recursive(item.children, depth + 1, result);
    }
}
