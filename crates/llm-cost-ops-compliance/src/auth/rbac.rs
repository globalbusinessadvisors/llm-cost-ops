// Role-Based Access Control (RBAC) system

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Resource types in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resource {
    /// Usage records
    Usage,
    /// Cost records
    Cost,
    /// Pricing tables
    Pricing,
    /// API keys
    ApiKey,
    /// Users
    User,
    /// Roles
    Role,
    /// Audit logs
    AuditLog,
    /// Forecasts
    Forecast,
    /// Budget
    Budget,
    /// Organization
    Organization,
    /// System settings
    System,
}

/// Actions that can be performed on resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// Read/view resource
    Read,
    /// Create new resource
    Create,
    /// Update existing resource
    Update,
    /// Delete resource
    Delete,
    /// List resources
    List,
    /// Execute special operations
    Execute,
    /// Export data
    Export,
    /// Import data
    Import,
    /// Manage permissions
    ManagePermissions,
}

/// Permission definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    /// Resource type
    pub resource: Resource,

    /// Action allowed
    pub action: Action,

    /// Optional scope restriction (e.g., organization_id)
    pub scope: Option<String>,
}

impl Permission {
    /// Create a new permission
    pub fn new(resource: Resource, action: Action) -> Self {
        Self {
            resource,
            action,
            scope: None,
        }
    }

    /// Create a scoped permission
    pub fn scoped(resource: Resource, action: Action, scope: String) -> Self {
        Self {
            resource,
            action,
            scope: Some(scope),
        }
    }

    /// Check if this permission matches another (considering scope)
    pub fn matches(&self, other: &Permission) -> bool {
        self.resource == other.resource
            && self.action == other.action
            && (self.scope.is_none() || self.scope == other.scope)
    }
}

/// Predefined system roles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleType {
    /// Super administrator with all permissions
    SuperAdmin,

    /// Organization administrator
    OrgAdmin,

    /// Organization member with read/write access
    OrgMember,

    /// Read-only user
    ReadOnly,

    /// Billing/finance user
    Billing,

    /// API-only user (service accounts)
    ApiUser,

    /// Auditor with read access to audit logs
    Auditor,

    /// Custom role
    Custom,
}

/// Role definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Unique role identifier
    pub id: String,

    /// Role name
    pub name: String,

    /// Role type
    pub role_type: RoleType,

    /// Description
    pub description: String,

    /// Set of permissions
    pub permissions: HashSet<Permission>,

    /// Whether this is a system-defined role
    pub is_system: bool,
}

impl Role {
    /// Create a new custom role
    pub fn new(id: String, name: String, description: String) -> Self {
        Self {
            id,
            name,
            role_type: RoleType::Custom,
            description,
            permissions: HashSet::new(),
            is_system: false,
        }
    }

    /// Add a permission to the role
    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    /// Remove a permission from the role
    pub fn remove_permission(&mut self, permission: &Permission) {
        self.permissions.remove(permission);
    }

    /// Check if role has a specific permission
    pub fn has_permission(&self, required: &Permission) -> bool {
        self.permissions.iter().any(|p| p.matches(required))
    }

    /// Create a super admin role
    pub fn super_admin() -> Self {
        let mut role = Self {
            id: "super_admin".to_string(),
            name: "Super Administrator".to_string(),
            role_type: RoleType::SuperAdmin,
            description: "Full system access with all permissions".to_string(),
            permissions: HashSet::new(),
            is_system: true,
        };

        // Add all permissions
        for resource in [
            Resource::Usage, Resource::Cost, Resource::Pricing, Resource::ApiKey,
            Resource::User, Resource::Role, Resource::AuditLog, Resource::Forecast,
            Resource::Budget, Resource::Organization, Resource::System,
        ] {
            for action in [
                Action::Read, Action::Create, Action::Update, Action::Delete,
                Action::List, Action::Execute, Action::Export, Action::Import,
                Action::ManagePermissions,
            ] {
                role.add_permission(Permission::new(resource, action));
            }
        }

        role
    }

    /// Create an org admin role
    pub fn org_admin(org_id: String) -> Self {
        let mut role = Self {
            id: format!("org_admin_{}", org_id),
            name: "Organization Administrator".to_string(),
            role_type: RoleType::OrgAdmin,
            description: "Full access within organization".to_string(),
            permissions: HashSet::new(),
            is_system: true,
        };

        // Add org-scoped permissions
        for resource in [
            Resource::Usage, Resource::Cost, Resource::Pricing, Resource::User,
            Resource::Forecast, Resource::Budget,
        ] {
            for action in [
                Action::Read, Action::Create, Action::Update, Action::Delete, Action::List,
            ] {
                role.add_permission(Permission::scoped(resource, action, org_id.clone()));
            }
        }

        role
    }

    /// Create a read-only role
    pub fn read_only(org_id: String) -> Self {
        let mut role = Self {
            id: format!("read_only_{}", org_id),
            name: "Read Only".to_string(),
            role_type: RoleType::ReadOnly,
            description: "Read-only access to organization data".to_string(),
            permissions: HashSet::new(),
            is_system: true,
        };

        // Add read permissions only
        for resource in [
            Resource::Usage, Resource::Cost, Resource::Pricing,
            Resource::Forecast, Resource::Budget,
        ] {
            role.add_permission(Permission::scoped(resource, Action::Read, org_id.clone()));
            role.add_permission(Permission::scoped(resource, Action::List, org_id.clone()));
        }

        role
    }

    /// Create a billing role
    pub fn billing(org_id: String) -> Self {
        let mut role = Self {
            id: format!("billing_{}", org_id),
            name: "Billing".to_string(),
            role_type: RoleType::Billing,
            description: "Access to billing and cost data".to_string(),
            permissions: HashSet::new(),
            is_system: true,
        };

        // Add billing-related permissions
        for resource in [Resource::Cost, Resource::Pricing, Resource::Budget] {
            for action in [Action::Read, Action::List, Action::Export] {
                role.add_permission(Permission::scoped(resource, action, org_id.clone()));
            }
        }

        role
    }

    /// Create an auditor role
    pub fn auditor() -> Self {
        let mut role = Self {
            id: "auditor".to_string(),
            name: "Auditor".to_string(),
            role_type: RoleType::Auditor,
            description: "Read access to audit logs".to_string(),
            permissions: HashSet::new(),
            is_system: true,
        };

        role.add_permission(Permission::new(Resource::AuditLog, Action::Read));
        role.add_permission(Permission::new(Resource::AuditLog, Action::List));
        role.add_permission(Permission::new(Resource::AuditLog, Action::Export));

        role
    }
}

/// User role assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRole {
    /// User identifier
    pub user_id: String,

    /// Assigned roles
    pub roles: Vec<String>,

    /// Additional direct permissions (beyond roles)
    pub additional_permissions: HashSet<Permission>,
}

impl UserRole {
    /// Create a new user role assignment
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            roles: Vec::new(),
            additional_permissions: HashSet::new(),
        }
    }

    /// Assign a role to the user
    pub fn assign_role(&mut self, role_id: String) {
        if !self.roles.contains(&role_id) {
            self.roles.push(role_id);
        }
    }

    /// Remove a role from the user
    pub fn remove_role(&mut self, role_id: &str) {
        self.roles.retain(|r| r != role_id);
    }

    /// Add a direct permission
    pub fn add_permission(&mut self, permission: Permission) {
        self.additional_permissions.insert(permission);
    }
}

/// RBAC manager
pub struct RbacManager {
    /// Role definitions
    roles: Arc<RwLock<HashMap<String, Role>>>,

    /// User role assignments
    user_roles: Arc<RwLock<HashMap<String, UserRole>>>,
}

impl RbacManager {
    /// Create a new RBAC manager
    pub fn new() -> Self {
        let mut roles = HashMap::new();

        // Add system roles
        let super_admin = Role::super_admin();
        roles.insert(super_admin.id.clone(), super_admin);

        let auditor = Role::auditor();
        roles.insert(auditor.id.clone(), auditor);

        Self {
            roles: Arc::new(RwLock::new(roles)),
            user_roles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a role
    pub async fn create_role(&self, role: Role) -> Result<(), RbacError> {
        let mut roles = self.roles.write().await;

        if roles.contains_key(&role.id) {
            return Err(RbacError::RoleAlreadyExists(role.id));
        }

        roles.insert(role.id.clone(), role);
        Ok(())
    }

    /// Get a role by ID
    pub async fn get_role(&self, role_id: &str) -> Option<Role> {
        let roles = self.roles.read().await;
        roles.get(role_id).cloned()
    }

    /// Update a role
    pub async fn update_role(&self, role: Role) -> Result<(), RbacError> {
        let mut roles = self.roles.write().await;

        if let Some(existing) = roles.get(&role.id) {
            if existing.is_system && role.role_type != existing.role_type {
                return Err(RbacError::CannotModifySystemRole);
            }
        }

        roles.insert(role.id.clone(), role);
        Ok(())
    }

    /// Delete a role
    pub async fn delete_role(&self, role_id: &str) -> Result<(), RbacError> {
        let mut roles = self.roles.write().await;

        if let Some(role) = roles.get(role_id) {
            if role.is_system {
                return Err(RbacError::CannotDeleteSystemRole);
            }
        }

        roles.remove(role_id);
        Ok(())
    }

    /// List all roles
    pub async fn list_roles(&self) -> Vec<Role> {
        let roles = self.roles.read().await;
        roles.values().cloned().collect()
    }

    /// Assign role to user
    pub async fn assign_user_role(&self, user_id: &str, role_id: &str) -> Result<(), RbacError> {
        // Verify role exists
        {
            let roles = self.roles.read().await;
            if !roles.contains_key(role_id) {
                return Err(RbacError::RoleNotFound(role_id.to_string()));
            }
        }

        let mut user_roles = self.user_roles.write().await;
        let user_role = user_roles
            .entry(user_id.to_string())
            .or_insert_with(|| UserRole::new(user_id.to_string()));

        user_role.assign_role(role_id.to_string());
        Ok(())
    }

    /// Remove role from user
    pub async fn remove_user_role(&self, user_id: &str, role_id: &str) -> Result<(), RbacError> {
        let mut user_roles = self.user_roles.write().await;

        if let Some(user_role) = user_roles.get_mut(user_id) {
            user_role.remove_role(role_id);
        }

        Ok(())
    }

    /// Get user's roles
    pub async fn get_user_roles(&self, user_id: &str) -> Vec<Role> {
        let user_roles = self.user_roles.read().await;
        let roles = self.roles.read().await;

        if let Some(user_role) = user_roles.get(user_id) {
            user_role
                .roles
                .iter()
                .filter_map(|role_id| roles.get(role_id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Check if user has permission
    pub async fn check_permission(
        &self,
        user_id: &str,
        required: &Permission,
    ) -> bool {
        let user_roles = self.user_roles.read().await;
        let roles = self.roles.read().await;

        if let Some(user_role) = user_roles.get(user_id) {
            // Check direct permissions
            if user_role.additional_permissions.iter().any(|p| p.matches(required)) {
                return true;
            }

            // Check role permissions
            for role_id in &user_role.roles {
                if let Some(role) = roles.get(role_id) {
                    if role.has_permission(required) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Get all permissions for a user
    pub async fn get_user_permissions(&self, user_id: &str) -> HashSet<Permission> {
        let user_roles = self.user_roles.read().await;
        let roles = self.roles.read().await;

        let mut permissions = HashSet::new();

        if let Some(user_role) = user_roles.get(user_id) {
            // Add direct permissions
            permissions.extend(user_role.additional_permissions.iter().cloned());

            // Add role permissions
            for role_id in &user_role.roles {
                if let Some(role) = roles.get(role_id) {
                    permissions.extend(role.permissions.iter().cloned());
                }
            }
        }

        permissions
    }

    /// Grant direct permission to user
    pub async fn grant_permission(&self, user_id: &str, permission: Permission) {
        let mut user_roles = self.user_roles.write().await;
        let user_role = user_roles
            .entry(user_id.to_string())
            .or_insert_with(|| UserRole::new(user_id.to_string()));

        user_role.add_permission(permission);
    }
}

impl Default for RbacManager {
    fn default() -> Self {
        Self::new()
    }
}

/// RBAC errors
#[derive(Debug, thiserror::Error)]
pub enum RbacError {
    #[error("Role already exists: {0}")]
    RoleAlreadyExists(String),

    #[error("Role not found: {0}")]
    RoleNotFound(String),

    #[error("Cannot modify system role")]
    CannotModifySystemRole,

    #[error("Cannot delete system role")]
    CannotDeleteSystemRole,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Invalid role configuration")]
    InvalidConfiguration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_matching() {
        let p1 = Permission::new(Resource::Usage, Action::Read);
        let p2 = Permission::new(Resource::Usage, Action::Read);
        let p3 = Permission::scoped(Resource::Usage, Action::Read, "org1".to_string());

        assert!(p1.matches(&p2));
        assert!(p1.matches(&p3)); // Unscoped matches scoped
        assert!(!p3.matches(&p1)); // Scoped doesn't match unscoped
    }

    #[test]
    fn test_role_permissions() {
        let mut role = Role::new(
            "test".to_string(),
            "Test Role".to_string(),
            "Test".to_string(),
        );

        let perm = Permission::new(Resource::Usage, Action::Read);
        role.add_permission(perm.clone());

        assert!(role.has_permission(&perm));

        role.remove_permission(&perm);
        assert!(!role.has_permission(&perm));
    }

    #[test]
    fn test_super_admin_role() {
        let role = Role::super_admin();

        assert_eq!(role.role_type, RoleType::SuperAdmin);
        assert!(role.is_system);

        // Should have all permissions
        let perm = Permission::new(Resource::System, Action::ManagePermissions);
        assert!(role.has_permission(&perm));
    }

    #[test]
    fn test_org_admin_role() {
        let role = Role::org_admin("org1".to_string());

        assert_eq!(role.role_type, RoleType::OrgAdmin);

        // Should have scoped permissions
        let perm = Permission::scoped(Resource::Usage, Action::Create, "org1".to_string());
        assert!(role.has_permission(&perm));
    }

    #[test]
    fn test_read_only_role() {
        let role = Role::read_only("org1".to_string());

        let read_perm = Permission::scoped(Resource::Usage, Action::Read, "org1".to_string());
        assert!(role.has_permission(&read_perm));

        let write_perm = Permission::scoped(Resource::Usage, Action::Create, "org1".to_string());
        assert!(!role.has_permission(&write_perm));
    }

    #[tokio::test]
    async fn test_rbac_manager_create_role() {
        let manager = RbacManager::new();

        let role = Role::new(
            "custom".to_string(),
            "Custom Role".to_string(),
            "Custom".to_string(),
        );

        assert!(manager.create_role(role.clone()).await.is_ok());
        assert!(manager.create_role(role).await.is_err()); // Duplicate
    }

    #[tokio::test]
    async fn test_rbac_manager_assign_role() {
        let manager = RbacManager::new();

        // Assign super_admin role
        assert!(manager.assign_user_role("user1", "super_admin").await.is_ok());

        // Assign non-existent role
        assert!(manager.assign_user_role("user1", "nonexistent").await.is_err());
    }

    #[tokio::test]
    async fn test_rbac_manager_check_permission() {
        let manager = RbacManager::new();

        // Assign super_admin role to user1
        manager.assign_user_role("user1", "super_admin").await.unwrap();

        let perm = Permission::new(Resource::Usage, Action::Read);
        assert!(manager.check_permission("user1", &perm).await);

        // User without role
        assert!(!manager.check_permission("user2", &perm).await);
    }

    #[tokio::test]
    async fn test_rbac_manager_direct_permission() {
        let manager = RbacManager::new();

        let perm = Permission::new(Resource::Usage, Action::Read);
        manager.grant_permission("user1", perm.clone()).await;

        assert!(manager.check_permission("user1", &perm).await);
    }

    #[tokio::test]
    async fn test_rbac_manager_get_user_permissions() {
        let manager = RbacManager::new();

        manager.assign_user_role("user1", "auditor").await.unwrap();

        let permissions = manager.get_user_permissions("user1").await;
        assert!(!permissions.is_empty());

        // Should have audit log permissions
        let has_audit_read = permissions.iter().any(|p| {
            p.resource == Resource::AuditLog && p.action == Action::Read
        });
        assert!(has_audit_read);
    }

    #[tokio::test]
    async fn test_rbac_manager_cannot_delete_system_role() {
        let manager = RbacManager::new();

        let result = manager.delete_role("super_admin").await;
        assert!(matches!(result, Err(RbacError::CannotDeleteSystemRole)));
    }
}
