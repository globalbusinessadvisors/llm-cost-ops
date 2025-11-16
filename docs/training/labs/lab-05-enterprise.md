# Lab 5: Enterprise Integration

## Overview

Deploy LLM Cost Ops in enterprise environments with multi-tenancy, SSO, RBAC, audit logging, compliance controls, and production-grade integrations.

**Estimated Time:** 120-150 minutes

**Difficulty Level:** Advanced

## Learning Objectives

- Configure multi-tenant architecture
- Integrate Single Sign-On (SSO) authentication
- Implement Role-Based Access Control (RBAC)
- Set up comprehensive audit logging
- Configure compliance controls and data governance
- Integrate with existing enterprise systems
- Set up CI/CD pipelines
- Deploy to production environments
- Configure monitoring and alerting
- Implement disaster recovery

## Prerequisites

- [ ] Completed Labs 1-4
- [ ] Understanding of enterprise security concepts
- [ ] Familiarity with Docker and Kubernetes
- [ ] Basic knowledge of OAuth 2.0 and OIDC
- [ ] Access to enterprise infrastructure

## Part 1: Multi-Tenant Configuration

### Step 1.1: Multi-Tenant Architecture Setup

Create `multi_tenant_manager.py`:

```python
#!/usr/bin/env python3
"""
Multi-Tenant Management System
"""

from datetime import datetime
from typing import Dict, List, Optional
import uuid
import hashlib
import json

class TenantManager:
    """Manage multi-tenant organization structure"""

    def __init__(self):
        self.tenants = {}
        self.api_keys = {}

    def create_tenant(
        self,
        organization_name: str,
        admin_email: str,
        tier: str = "standard",
        limits: Optional[Dict] = None
    ) -> Dict:
        """Create a new tenant organization"""

        tenant_id = f"org-{str(uuid.uuid4())[:8]}"

        # Default limits based on tier
        default_limits = {
            "free": {
                "max_requests_per_day": 1000,
                "max_projects": 3,
                "max_users": 5,
                "max_monthly_cost": 100.00,
                "retention_days": 30
            },
            "standard": {
                "max_requests_per_day": 100000,
                "max_projects": 25,
                "max_users": 50,
                "max_monthly_cost": 10000.00,
                "retention_days": 90
            },
            "enterprise": {
                "max_requests_per_day": -1,  # unlimited
                "max_projects": -1,
                "max_users": -1,
                "max_monthly_cost": -1,
                "retention_days": 365
            }
        }

        tenant = {
            "tenant_id": tenant_id,
            "organization_name": organization_name,
            "admin_email": admin_email,
            "tier": tier,
            "limits": limits or default_limits.get(tier, default_limits["standard"]),
            "created_at": datetime.utcnow().isoformat(),
            "status": "active",
            "settings": {
                "data_region": "us-east-1",
                "encryption_enabled": True,
                "audit_logging": True,
                "sso_enabled": tier == "enterprise",
                "custom_domain": None
            }
        }

        self.tenants[tenant_id] = tenant

        print(f"✓ Tenant created: {organization_name}")
        print(f"  Tenant ID: {tenant_id}")
        print(f"  Tier: {tier}")
        print(f"  Limits: {json.dumps(tenant['limits'], indent=4)}")

        return tenant

    def generate_api_key(self, tenant_id: str, description: str = "Default API Key") -> Dict:
        """Generate API key for tenant"""

        if tenant_id not in self.tenants:
            raise ValueError(f"Tenant {tenant_id} not found")

        # Generate secure API key
        api_key = f"costops_{str(uuid.uuid4()).replace('-', '')}"

        # Hash for storage
        key_hash = hashlib.sha256(api_key.encode()).hexdigest()

        key_info = {
            "key_id": str(uuid.uuid4()),
            "tenant_id": tenant_id,
            "key_hash": key_hash,
            "description": description,
            "created_at": datetime.utcnow().isoformat(),
            "last_used": None,
            "status": "active",
            "scopes": ["read", "write"],
            "rate_limit": 1000  # requests per hour
        }

        self.api_keys[key_info["key_id"]] = key_info

        print(f"✓ API Key generated for {self.tenants[tenant_id]['organization_name']}")
        print(f"  Key: {api_key}")
        print(f"  IMPORTANT: Save this key securely. It won't be shown again.")

        return {
            "api_key": api_key,
            "key_id": key_info["key_id"]
        }

    def create_project(self, tenant_id: str, project_name: str, budget: float) -> Dict:
        """Create project within tenant"""

        if tenant_id not in self.tenants:
            raise ValueError(f"Tenant {tenant_id} not found")

        tenant = self.tenants[tenant_id]

        # Check limits
        current_projects = sum(1 for t in self.tenants.values()
                              if t.get("parent_tenant_id") == tenant_id)

        max_projects = tenant["limits"]["max_projects"]
        if max_projects != -1 and current_projects >= max_projects:
            raise ValueError(f"Project limit reached ({max_projects})")

        project_id = f"proj-{str(uuid.uuid4())[:8]}"

        project = {
            "project_id": project_id,
            "tenant_id": tenant_id,
            "name": project_name,
            "budget": budget,
            "created_at": datetime.utcnow().isoformat(),
            "status": "active",
            "tags": []
        }

        # Store project (in real implementation, this would go to database)
        print(f"✓ Project created: {project_name}")
        print(f"  Project ID: {project_id}")
        print(f"  Budget: ${budget:,.2f}")

        return project

    def enforce_tenant_isolation(self, tenant_id: str, resource_id: str) -> bool:
        """Verify resource belongs to tenant (data isolation)"""

        # In real implementation, this would check database
        # to ensure resource belongs to tenant

        # Example validation
        if not resource_id.startswith(f"{tenant_id}_"):
            print(f"⚠️  Access denied: Resource {resource_id} does not belong to tenant {tenant_id}")
            return False

        return True

    def get_tenant_usage(self, tenant_id: str) -> Dict:
        """Get tenant usage statistics"""

        if tenant_id not in self.tenants:
            raise ValueError(f"Tenant {tenant_id} not found")

        tenant = self.tenants[tenant_id]

        # In real implementation, query database for actual usage
        usage = {
            "tenant_id": tenant_id,
            "organization": tenant["organization_name"],
            "period": "current_month",
            "requests": 45000,
            "total_cost": 2500.00,
            "projects": 12,
            "users": 25,
            "limits": tenant["limits"],
            "utilization": {
                "requests": 45.0,  # 45% of daily limit
                "projects": 48.0,  # 48% of project limit
                "users": 50.0,     # 50% of user limit
                "cost": 25.0        # 25% of cost limit
            }
        }

        return usage

    def display_tenant_info(self, tenant_id: str):
        """Display tenant information"""

        if tenant_id not in self.tenants:
            print(f"Tenant {tenant_id} not found")
            return

        tenant = self.tenants[tenant_id]
        usage = self.get_tenant_usage(tenant_id)

        print("=" * 80)
        print(f"TENANT INFORMATION: {tenant['organization_name']}")
        print("=" * 80)

        print(f"\n📋 Details:")
        print(f"  Tenant ID: {tenant['tenant_id']}")
        print(f"  Admin: {tenant['admin_email']}")
        print(f"  Tier: {tenant['tier'].upper()}")
        print(f"  Status: {tenant['status'].upper()}")
        print(f"  Created: {tenant['created_at']}")

        print(f"\n⚙️  Settings:")
        for key, value in tenant['settings'].items():
            print(f"  {key}: {value}")

        print(f"\n📊 Usage:")
        print(f"  Requests: {usage['requests']:,} / {tenant['limits']['max_requests_per_day']:,} daily")
        print(f"  Projects: {usage['projects']} / {tenant['limits']['max_projects']}")
        print(f"  Users: {usage['users']} / {tenant['limits']['max_users']}")
        print(f"  Monthly Cost: ${usage['total_cost']:,.2f} / ${tenant['limits']['max_monthly_cost']:,.2f}")

        print(f"\n📈 Utilization:")
        for resource, percent in usage['utilization'].items():
            bar_length = int(percent / 2)
            bar = '█' * bar_length + '░' * (50 - bar_length)
            print(f"  {resource.capitalize():<12} [{bar}] {percent:.1f}%")

        print("\n" + "=" * 80)


# Example usage
if __name__ == "__main__":
    manager = TenantManager()

    # Create tenants
    print("Creating Tenants...")
    print("=" * 80 + "\n")

    acme = manager.create_tenant(
        organization_name="Acme Corporation",
        admin_email="admin@acme.com",
        tier="enterprise"
    )

    print("\n")

    startup = manager.create_tenant(
        organization_name="Startup Inc",
        admin_email="founder@startup.com",
        tier="standard"
    )

    # Generate API keys
    print("\n\nGenerating API Keys...")
    print("=" * 80 + "\n")

    acme_key = manager.generate_api_key(acme["tenant_id"], "Production API Key")
    startup_key = manager.generate_api_key(startup["tenant_id"], "Development API Key")

    # Create projects
    print("\n\nCreating Projects...")
    print("=" * 80 + "\n")

    manager.create_project(acme["tenant_id"], "Customer Support Bot", 5000.00)
    manager.create_project(acme["tenant_id"], "Content Generation", 3000.00)
    manager.create_project(startup["tenant_id"], "MVP Chatbot", 500.00)

    # Display tenant info
    print("\n")
    manager.display_tenant_info(acme["tenant_id"])
```

Run the multi-tenant manager:

```bash
python multi_tenant_manager.py
```

## Part 2: SSO Integration

### Step 2.1: OIDC/SAML Authentication

Create `sso_integration.py`:

```python
#!/usr/bin/env python3
"""
SSO Integration (OIDC/SAML)
"""

from datetime import datetime, timedelta
from typing import Dict, Optional
import jwt
import hashlib
import json

class SSOManager:
    """Manage SSO authentication"""

    def __init__(self, provider: str = "oidc"):
        self.provider = provider
        self.secret_key = "your-secret-key-change-in-production"

    def configure_oidc(self, config: Dict) -> Dict:
        """Configure OpenID Connect provider"""

        oidc_config = {
            "provider": "oidc",
            "client_id": config.get("client_id"),
            "client_secret": config.get("client_secret"),
            "issuer": config.get("issuer"),
            "authorization_endpoint": f"{config.get('issuer')}/authorize",
            "token_endpoint": f"{config.get('issuer')}/token",
            "userinfo_endpoint": f"{config.get('issuer')}/userinfo",
            "jwks_uri": f"{config.get('issuer')}/keys",
            "scopes": ["openid", "profile", "email"],
            "redirect_uri": config.get("redirect_uri", "https://costops.example.com/auth/callback")
        }

        print("✓ OIDC Configuration:")
        print(json.dumps(oidc_config, indent=2))

        return oidc_config

    def configure_saml(self, config: Dict) -> Dict:
        """Configure SAML provider"""

        saml_config = {
            "provider": "saml",
            "entity_id": config.get("entity_id"),
            "sso_url": config.get("sso_url"),
            "x509_cert": config.get("x509_cert"),
            "name_id_format": "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress",
            "want_assertions_signed": True,
            "want_messages_signed": True
        }

        print("✓ SAML Configuration:")
        print(json.dumps({k: v for k, v in saml_config.items() if k != 'x509_cert'}, indent=2))

        return saml_config

    def authenticate_user(self, id_token: str) -> Dict:
        """Authenticate user from SSO provider"""

        try:
            # Decode and verify JWT token
            # In production, verify with provider's public key
            payload = jwt.decode(id_token, self.secret_key, algorithms=["HS256"])

            user = {
                "user_id": payload.get("sub"),
                "email": payload.get("email"),
                "name": payload.get("name"),
                "tenant_id": payload.get("tenant_id"),
                "roles": payload.get("roles", []),
                "authenticated_at": datetime.utcnow().isoformat(),
                "authentication_method": "sso"
            }

            print(f"✓ User authenticated: {user['email']}")
            return user

        except jwt.InvalidTokenError as e:
            print(f"✗ Authentication failed: {e}")
            return None

    def create_session(self, user: Dict) -> str:
        """Create authenticated session"""

        session_data = {
            "user_id": user["user_id"],
            "tenant_id": user["tenant_id"],
            "roles": user["roles"],
            "created_at": datetime.utcnow().isoformat(),
            "expires_at": (datetime.utcnow() + timedelta(hours=8)).isoformat()
        }

        # Create JWT session token
        session_token = jwt.encode(session_data, self.secret_key, algorithm="HS256")

        print(f"✓ Session created for {user['email']}")
        print(f"  Expires: {session_data['expires_at']}")

        return session_token

    def verify_session(self, session_token: str) -> Optional[Dict]:
        """Verify session token"""

        try:
            session_data = jwt.decode(session_token, self.secret_key, algorithms=["HS256"])

            # Check expiration
            expires_at = datetime.fromisoformat(session_data["expires_at"])
            if datetime.utcnow() > expires_at:
                print("✗ Session expired")
                return None

            return session_data

        except jwt.InvalidTokenError:
            print("✗ Invalid session token")
            return None


# Example usage
if __name__ == "__main__":
    sso = SSOManager()

    # Configure OIDC
    print("Configuring OIDC Provider...")
    print("=" * 80 + "\n")

    oidc_config = sso.configure_oidc({
        "client_id": "costops-client",
        "client_secret": "client-secret",
        "issuer": "https://auth.example.com",
        "redirect_uri": "https://costops.example.com/auth/callback"
    })

    # Simulate authentication
    print("\n\nSimulating User Authentication...")
    print("=" * 80 + "\n")

    # Create test ID token
    test_id_token = jwt.encode({
        "sub": "user-123",
        "email": "alice@acme.com",
        "name": "Alice Smith",
        "tenant_id": "org-acme-corp",
        "roles": ["admin", "cost_analyst"]
    }, sso.secret_key, algorithm="HS256")

    # Authenticate
    user = sso.authenticate_user(test_id_token)

    if user:
        # Create session
        session_token = sso.create_session(user)

        # Verify session
        print("\n\nVerifying Session...")
        print("=" * 80 + "\n")

        session = sso.verify_session(session_token)
        if session:
            print(f"✓ Session valid:")
            print(json.dumps(session, indent=2))
```

## Part 3: Role-Based Access Control (RBAC)

Create `rbac_system.py`:

```python
#!/usr/bin/env python3
"""
Role-Based Access Control System
"""

from typing import Dict, List, Set
from enum import Enum

class Permission(Enum):
    """System permissions"""
    # Cost data
    VIEW_COSTS = "view_costs"
    EXPORT_COSTS = "export_costs"

    # Budget management
    VIEW_BUDGETS = "view_budgets"
    CREATE_BUDGETS = "create_budgets"
    EDIT_BUDGETS = "edit_budgets"
    DELETE_BUDGETS = "delete_budgets"

    # Analytics
    VIEW_ANALYTICS = "view_analytics"
    CREATE_REPORTS = "create_reports"

    # Administration
    MANAGE_USERS = "manage_users"
    MANAGE_PROJECTS = "manage_projects"
    MANAGE_API_KEYS = "manage_api_keys"
    VIEW_AUDIT_LOGS = "view_audit_logs"

    # System
    SYSTEM_ADMIN = "system_admin"

class Role:
    """Role definition with permissions"""

    def __init__(self, name: str, permissions: Set[Permission], description: str = ""):
        self.name = name
        self.permissions = permissions
        self.description = description

class RBACManager:
    """Manage roles and permissions"""

    def __init__(self):
        self.roles = self._define_default_roles()
        self.user_roles = {}

    def _define_default_roles(self) -> Dict[str, Role]:
        """Define default system roles"""

        return {
            "viewer": Role(
                name="Viewer",
                description="Read-only access to costs and analytics",
                permissions={
                    Permission.VIEW_COSTS,
                    Permission.VIEW_BUDGETS,
                    Permission.VIEW_ANALYTICS
                }
            ),
            "analyst": Role(
                name="Analyst",
                description="Can view and export data, create reports",
                permissions={
                    Permission.VIEW_COSTS,
                    Permission.EXPORT_COSTS,
                    Permission.VIEW_BUDGETS,
                    Permission.VIEW_ANALYTICS,
                    Permission.CREATE_REPORTS
                }
            ),
            "budget_manager": Role(
                name="Budget Manager",
                description="Manage budgets and view costs",
                permissions={
                    Permission.VIEW_COSTS,
                    Permission.VIEW_BUDGETS,
                    Permission.CREATE_BUDGETS,
                    Permission.EDIT_BUDGETS,
                    Permission.DELETE_BUDGETS,
                    Permission.VIEW_ANALYTICS
                }
            ),
            "project_admin": Role(
                name="Project Admin",
                description="Manage projects and their budgets",
                permissions={
                    Permission.VIEW_COSTS,
                    Permission.EXPORT_COSTS,
                    Permission.VIEW_BUDGETS,
                    Permission.CREATE_BUDGETS,
                    Permission.EDIT_BUDGETS,
                    Permission.MANAGE_PROJECTS,
                    Permission.VIEW_ANALYTICS,
                    Permission.CREATE_REPORTS
                }
            ),
            "admin": Role(
                name="Administrator",
                description="Full administrative access",
                permissions={
                    Permission.VIEW_COSTS,
                    Permission.EXPORT_COSTS,
                    Permission.VIEW_BUDGETS,
                    Permission.CREATE_BUDGETS,
                    Permission.EDIT_BUDGETS,
                    Permission.DELETE_BUDGETS,
                    Permission.VIEW_ANALYTICS,
                    Permission.CREATE_REPORTS,
                    Permission.MANAGE_USERS,
                    Permission.MANAGE_PROJECTS,
                    Permission.MANAGE_API_KEYS,
                    Permission.VIEW_AUDIT_LOGS
                }
            ),
            "system_admin": Role(
                name="System Administrator",
                description="Complete system access",
                permissions=set(Permission)  # All permissions
            )
        }

    def assign_role(self, user_id: str, tenant_id: str, role_name: str):
        """Assign role to user"""

        if role_name not in self.roles:
            raise ValueError(f"Role '{role_name}' not found")

        key = f"{tenant_id}:{user_id}"
        self.user_roles[key] = role_name

        print(f"✓ Assigned role '{role_name}' to user {user_id} in tenant {tenant_id}")

    def check_permission(self, user_id: str, tenant_id: str, permission: Permission) -> bool:
        """Check if user has permission"""

        key = f"{tenant_id}:{user_id}"
        role_name = self.user_roles.get(key)

        if not role_name:
            return False

        role = self.roles.get(role_name)
        if not role:
            return False

        return permission in role.permissions

    def get_user_permissions(self, user_id: str, tenant_id: str) -> Set[Permission]:
        """Get all permissions for user"""

        key = f"{tenant_id}:{user_id}"
        role_name = self.user_roles.get(key)

        if not role_name:
            return set()

        role = self.roles.get(role_name)
        return role.permissions if role else set()

    def display_roles(self):
        """Display all available roles"""

        print("=" * 80)
        print("AVAILABLE ROLES")
        print("=" * 80)

        for role_name, role in self.roles.items():
            print(f"\n{role.name} ({role_name})")
            print(f"  {role.description}")
            print(f"  Permissions ({len(role.permissions)}):")
            for perm in sorted(role.permissions, key=lambda p: p.value):
                print(f"    • {perm.value}")

        print("\n" + "=" * 80)

    def audit_access_attempt(
        self,
        user_id: str,
        tenant_id: str,
        action: str,
        permission: Permission,
        resource: str
    ):
        """Log access attempt for audit"""

        has_permission = self.check_permission(user_id, tenant_id, permission)

        audit_entry = {
            "timestamp": datetime.utcnow().isoformat(),
            "user_id": user_id,
            "tenant_id": tenant_id,
            "action": action,
            "permission": permission.value,
            "resource": resource,
            "result": "ALLOWED" if has_permission else "DENIED"
        }

        # In production, log to audit system
        print(f"{'✓' if has_permission else '✗'} {audit_entry['result']}: "
              f"{user_id} attempted {action} on {resource}")

        return has_permission


# Example usage
if __name__ == "__main__":
    rbac = RBACManager()

    # Display available roles
    rbac.display_roles()

    # Assign roles
    print("\n\nAssigning Roles...")
    print("=" * 80 + "\n")

    rbac.assign_role("user-alice", "org-acme", "admin")
    rbac.assign_role("user-bob", "org-acme", "analyst")
    rbac.assign_role("user-charlie", "org-acme", "viewer")

    # Test permissions
    print("\n\nTesting Permissions...")
    print("=" * 80 + "\n")

    # Alice (admin) can delete budgets
    rbac.audit_access_attempt(
        "user-alice", "org-acme",
        "delete", Permission.DELETE_BUDGETS,
        "budget-2025-q1"
    )

    # Bob (analyst) cannot delete budgets
    rbac.audit_access_attempt(
        "user-bob", "org-acme",
        "delete", Permission.DELETE_BUDGETS,
        "budget-2025-q1"
    )

    # Charlie (viewer) can view costs
    rbac.audit_access_attempt(
        "user-charlie", "org-acme",
        "view", Permission.VIEW_COSTS,
        "cost-records"
    )

    # Charlie (viewer) cannot export
    rbac.audit_access_attempt(
        "user-charlie", "org-acme",
        "export", Permission.EXPORT_COSTS,
        "cost-records"
    )
```

## Part 4: Audit Logging

Create `audit_logger.py`:

```python
#!/usr/bin/env python3
"""
Comprehensive Audit Logging
"""

from datetime import datetime
from typing import Dict, Optional, List
import json
from enum import Enum

class AuditEventType(Enum):
    """Audit event types"""
    # Authentication
    LOGIN = "auth.login"
    LOGOUT = "auth.logout"
    LOGIN_FAILED = "auth.login_failed"

    # Budget operations
    BUDGET_CREATED = "budget.created"
    BUDGET_UPDATED = "budget.updated"
    BUDGET_DELETED = "budget.deleted"
    BUDGET_ALERT = "budget.alert"

    # Data operations
    COST_VIEWED = "cost.viewed"
    COST_EXPORTED = "cost.exported"
    REPORT_GENERATED = "report.generated"

    # Administrative
    USER_CREATED = "user.created"
    USER_UPDATED = "user.updated"
    USER_DELETED = "user.deleted"
    ROLE_ASSIGNED = "role.assigned"
    API_KEY_CREATED = "apikey.created"
    API_KEY_REVOKED = "apikey.revoked"

    # Security
    ACCESS_DENIED = "security.access_denied"
    SUSPICIOUS_ACTIVITY = "security.suspicious"

class AuditLogger:
    """Comprehensive audit logging system"""

    def __init__(self):
        self.logs = []

    def log_event(
        self,
        event_type: AuditEventType,
        user_id: str,
        tenant_id: str,
        details: Dict,
        ip_address: Optional[str] = None,
        user_agent: Optional[str] = None
    ):
        """Log audit event"""

        event = {
            "event_id": str(uuid.uuid4()),
            "timestamp": datetime.utcnow().isoformat(),
            "event_type": event_type.value,
            "user_id": user_id,
            "tenant_id": tenant_id,
            "details": details,
            "ip_address": ip_address or "unknown",
            "user_agent": user_agent or "unknown",
            "severity": self._determine_severity(event_type)
        }

        self.logs.append(event)

        # In production, send to centralized logging system
        # (e.g., Elasticsearch, CloudWatch, Datadog)

        print(f"[AUDIT] {event['timestamp']} | {event_type.value} | {user_id} | {tenant_id}")

        return event

    def _determine_severity(self, event_type: AuditEventType) -> str:
        """Determine event severity"""

        high_severity = {
            AuditEventType.LOGIN_FAILED,
            AuditEventType.ACCESS_DENIED,
            AuditEventType.SUSPICIOUS_ACTIVITY,
            AuditEventType.USER_DELETED,
            AuditEventType.API_KEY_REVOKED
        }

        medium_severity = {
            AuditEventType.BUDGET_DELETED,
            AuditEventType.USER_CREATED,
            AuditEventType.ROLE_ASSIGNED
        }

        if event_type in high_severity:
            return "HIGH"
        elif event_type in medium_severity:
            return "MEDIUM"
        else:
            return "LOW"

    def query_logs(
        self,
        tenant_id: Optional[str] = None,
        user_id: Optional[str] = None,
        event_type: Optional[AuditEventType] = None,
        start_date: Optional[str] = None,
        end_date: Optional[str] = None
    ) -> List[Dict]:
        """Query audit logs"""

        results = self.logs

        if tenant_id:
            results = [log for log in results if log['tenant_id'] == tenant_id]

        if user_id:
            results = [log for log in results if log['user_id'] == user_id]

        if event_type:
            results = [log for log in results if log['event_type'] == event_type.value]

        if start_date:
            results = [log for log in results if log['timestamp'] >= start_date]

        if end_date:
            results = [log for log in results if log['timestamp'] <= end_date]

        return results

    def generate_audit_report(self, tenant_id: str, days: int = 30):
        """Generate audit report"""

        from collections import Counter

        print("=" * 80)
        print(f"AUDIT REPORT - {tenant_id}")
        print("=" * 80)

        logs = self.query_logs(tenant_id=tenant_id)

        # Event type distribution
        event_types = Counter(log['event_type'] for log in logs)

        print(f"\n📊 Event Distribution:")
        for event_type, count in event_types.most_common(10):
            print(f"  {event_type:<30} {count:>5} events")

        # User activity
        user_activity = Counter(log['user_id'] for log in logs)

        print(f"\n👥 Most Active Users:")
        for user_id, count in user_activity.most_common(5):
            print(f"  {user_id:<30} {count:>5} events")

        # Security events
        security_events = [log for log in logs
                          if log['severity'] == 'HIGH']

        print(f"\n🔒 Security Events:")
        print(f"  Total high-severity events: {len(security_events)}")

        if security_events:
            print(f"\n  Recent high-severity events:")
            for event in security_events[-5:]:
                print(f"    {event['timestamp']} | {event['event_type']} | {event['user_id']}")

        print("\n" + "=" * 80)


# Example usage
if __name__ == "__main__":
    import uuid

    auditor = AuditLogger()

    # Simulate various events
    print("Logging Audit Events...")
    print("=" * 80 + "\n")

    # Successful login
    auditor.log_event(
        AuditEventType.LOGIN,
        "user-alice",
        "org-acme",
        {"method": "sso", "provider": "okta"},
        ip_address="192.168.1.100"
    )

    # Budget created
    auditor.log_event(
        AuditEventType.BUDGET_CREATED,
        "user-alice",
        "org-acme",
        {"budget_id": "budget-123", "limit": 5000.00}
    )

    # Failed login
    auditor.log_event(
        AuditEventType.LOGIN_FAILED,
        "unknown",
        "org-acme",
        {"reason": "invalid_credentials"},
        ip_address="203.0.113.45"
    )

    # Access denied
    auditor.log_event(
        AuditEventType.ACCESS_DENIED,
        "user-bob",
        "org-acme",
        {"resource": "budget-123", "action": "delete", "reason": "insufficient_permissions"}
    )

    # Generate report
    print("\n")
    auditor.generate_audit_report("org-acme")
```

## Part 5: Production Deployment

### Kubernetes Deployment Configuration

Create `k8s-production.yaml`:

```yaml
# Production Kubernetes Deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llm-cost-ops-prod
  namespace: cost-ops-production
  labels:
    app: llm-cost-ops
    environment: production
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: llm-cost-ops
  template:
    metadata:
      labels:
        app: llm-cost-ops
        version: v1.0.0
    spec:
      serviceAccountName: cost-ops-sa
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 2000
      containers:
      - name: cost-ops-api
        image: your-registry.com/llm-cost-ops:v1.0.0
        ports:
        - containerPort: 8080
          name: http
          protocol: TCP
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: cost-ops-secrets
              key: database-url
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: cost-ops-secrets
              key: jwt-secret
        - name: ENVIRONMENT
          value: "production"
        - name: LOG_LEVEL
          value: "info"
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        securityContext:
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          capabilities:
            drop:
            - ALL
---
apiVersion: v1
kind: Service
metadata:
  name: llm-cost-ops-svc
  namespace: cost-ops-production
spec:
  type: LoadBalancer
  selector:
    app: llm-cost-ops
  ports:
  - port: 443
    targetPort: 8080
    protocol: TCP
    name: https
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: cost-ops-hpa
  namespace: cost-ops-production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: llm-cost-ops-prod
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

## Exercises and Challenges

### Exercise 1: Multi-Region Deployment
Configure multi-region deployment with data residency requirements.

### Exercise 2: Advanced RBAC
Implement custom roles with fine-grained permissions for specific resources.

### Exercise 3: Compliance Dashboard
Build a compliance dashboard showing GDPR, SOC2, and HIPAA readiness.

### Exercise 4: Disaster Recovery
Implement automated backup and disaster recovery procedures.

### Exercise 5: Integration Pipeline
Create a complete CI/CD pipeline with automated testing and deployment.

## Review Questions

1. What are the key components of a multi-tenant architecture?
2. How does SSO improve security in enterprise deployments?
3. What is the difference between RBAC and ABAC?
4. Why is audit logging critical for compliance?
5. What are the key considerations for production deployment?

## Next Steps

Congratulations on completing all 5 labs! You now have comprehensive knowledge of:
- Basic cost tracking
- Analytics and reporting
- Budget management
- Cost optimization
- Enterprise integration

### Recommended Next Steps:
1. Deploy to your own infrastructure
2. Integrate with your existing systems
3. Customize for your specific use cases
4. Contribute to the open-source project

---

**End of Lab 5 - Training Complete!**
