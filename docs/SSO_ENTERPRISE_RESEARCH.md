# Enterprise SSO: Standards & Research Report

**Document Version:** 1.0.0
**Date:** 2025-11-15
**Author:** SSO Standards & Research Specialist
**Status:** Final Research Report

---

## Executive Summary

This document provides comprehensive research on enterprise Single Sign-On (SSO) requirements, standards, and best practices for the LLM-CostOps platform. The research covers industry-standard protocols (SAML 2.0, OAuth 2.0, OpenID Connect, SCIM 2.0), major identity provider integrations, security requirements, and enterprise features necessary for production-grade SSO implementation.

**Key Findings:**
- Current auth implementation uses JWT + RBAC, providing foundation for SSO integration
- SAML 2.0 and OpenID Connect are essential for enterprise adoption
- 5+ major IdP integrations required: Okta, Auth0, Azure AD, Google Workspace, GitHub
- Just-in-Time (JIT) provisioning critical for seamless user onboarding
- SOC 2 and GDPR compliance requirements mandate specific security controls

---

## Table of Contents

1. [SSO Protocol Standards](#1-sso-protocol-standards)
2. [Identity Provider Integration](#2-identity-provider-integration)
3. [Security Requirements](#3-security-requirements)
4. [Enterprise Features](#4-enterprise-features)
5. [User Provisioning Strategies](#5-user-provisioning-strategies)
6. [Session Management](#6-session-management)
7. [Compliance Requirements](#7-compliance-requirements)
8. [Implementation Roadmap](#8-implementation-roadmap)
9. [References & Resources](#9-references--resources)

---

## 1. SSO Protocol Standards

### 1.1 SAML 2.0 (Security Assertion Markup Language)

#### Overview
SAML 2.0 is the industry-standard XML-based protocol for enterprise SSO, widely adopted by large organizations and traditional enterprise IdPs.

#### Core Components

**Assertions**
- **Authentication Assertion**: Proves user authenticated successfully
- **Attribute Assertion**: Contains user profile information (email, name, groups)
- **Authorization Assertion**: Contains permissions and roles

```xml
<!-- Example SAML 2.0 Assertion Structure -->
<saml:Assertion xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion"
                 ID="_abc123"
                 IssueInstant="2025-11-15T10:00:00Z"
                 Version="2.0">
  <saml:Issuer>https://idp.example.com</saml:Issuer>

  <saml:Subject>
    <saml:NameID Format="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress">
      user@example.com
    </saml:NameID>
    <saml:SubjectConfirmation Method="urn:oasis:names:tc:SAML:2.0:cm:bearer">
      <saml:SubjectConfirmationData
        NotOnOrAfter="2025-11-15T10:05:00Z"
        Recipient="https://llm-cost-ops.example.com/saml/acs"/>
    </saml:SubjectConfirmation>
  </saml:Subject>

  <saml:Conditions NotBefore="2025-11-15T10:00:00Z"
                   NotOnOrAfter="2025-11-15T10:05:00Z">
    <saml:AudienceRestriction>
      <saml:Audience>llm-cost-ops</saml:Audience>
    </saml:AudienceRestriction>
  </saml:Conditions>

  <saml:AuthnStatement AuthnInstant="2025-11-15T10:00:00Z"
                       SessionIndex="_session123">
    <saml:AuthnContext>
      <saml:AuthnContextClassRef>
        urn:oasis:names:tc:SAML:2.0:ac:classes:PasswordProtectedTransport
      </saml:AuthnContextClassRef>
    </saml:AuthnContext>
  </saml:AuthnStatement>

  <saml:AttributeStatement>
    <saml:Attribute Name="email">
      <saml:AttributeValue>user@example.com</saml:AttributeValue>
    </saml:Attribute>
    <saml:Attribute Name="firstName">
      <saml:AttributeValue>John</saml:AttributeValue>
    </saml:Attribute>
    <saml:Attribute Name="lastName">
      <saml:AttributeValue>Doe</saml:AttributeValue>
    </saml:Attribute>
    <saml:Attribute Name="groups">
      <saml:AttributeValue>org_admin</saml:AttributeValue>
      <saml:AttributeValue>billing</saml:AttributeValue>
    </saml:Attribute>
  </saml:AttributeStatement>
</saml:Assertion>
```

**Bindings**
- **HTTP Redirect Binding**: AuthnRequest via URL redirect (GET)
- **HTTP POST Binding**: Response via form POST (most common)
- **HTTP Artifact Binding**: Response reference retrieved via back-channel

**Profiles**
- **Web Browser SSO Profile**: Primary profile for web applications
- **Single Logout Profile**: Coordinate logout across multiple SPs
- **Enhanced Client or Proxy (ECP) Profile**: For non-browser clients

#### SAML 2.0 Flow (SP-Initiated)

```
┌─────────┐                 ┌──────────────┐                 ┌─────────┐
│ User    │                 │ LLM-CostOps  │                 │   IdP   │
│ Browser │                 │     (SP)     │                 │         │
└────┬────┘                 └──────┬───────┘                 └────┬────┘
     │                              │                              │
     │  1. Access Protected Resource│                              │
     ├─────────────────────────────>│                              │
     │                              │                              │
     │  2. Redirect to IdP (AuthnRequest)                          │
     │<─────────────────────────────┤                              │
     │                              │                              │
     │  3. Forward AuthnRequest     │                              │
     ├──────────────────────────────┼─────────────────────────────>│
     │                              │                              │
     │  4. Authenticate User        │                              │
     │<─────────────────────────────┼──────────────────────────────┤
     │                              │                              │
     │  5. SAML Response (POST)     │                              │
     ├─────────────────────────────>│                              │
     │                              │                              │
     │                              │  6. Validate Assertion       │
     │                              │  - Check signature           │
     │                              │  - Verify timestamps         │
     │                              │  - Validate audience         │
     │                              │                              │
     │  7. Create Session & Redirect│                              │
     │<─────────────────────────────┤                              │
     │                              │                              │
```

#### Implementation Requirements

**MUST Implement:**
- XML signature verification (RSA-SHA256 minimum)
- Timestamp validation (NotBefore, NotOnOrAfter)
- Audience restriction validation
- Assertion replay prevention (track assertion IDs)
- DEFLATE compression for AuthnRequest
- Support for both redirect and POST bindings

**SHOULD Implement:**
- Single Logout (SLO) support
- Name ID Management
- Assertion encryption for sensitive attributes
- IdP-initiated SSO flow

**Metadata Management:**
```xml
<!-- SP Metadata Example -->
<EntityDescriptor xmlns="urn:oasis:names:tc:SAML:2.0:metadata"
                  entityID="llm-cost-ops">
  <SPSSODescriptor
      AuthnRequestsSigned="true"
      WantAssertionsSigned="true"
      protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">

    <KeyDescriptor use="signing">
      <KeyInfo xmlns="http://www.w3.org/2000/09/xmldsig#">
        <X509Data>
          <X509Certificate>MIIDdDCCAly...</X509Certificate>
        </X509Data>
      </KeyInfo>
    </KeyDescriptor>

    <SingleLogoutService
        Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
        Location="https://llm-cost-ops.example.com/saml/sls"/>

    <AssertionConsumerService
        Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
        Location="https://llm-cost-ops.example.com/saml/acs"
        index="0" isDefault="true"/>
  </SPSSODescriptor>
</EntityDescriptor>
```

---

### 1.2 OAuth 2.0 & OpenID Connect (OIDC)

#### OAuth 2.0 Authorization Framework

OAuth 2.0 provides delegated authorization, while OpenID Connect adds an identity layer on top for authentication.

#### Authorization Code Flow with PKCE

**PKCE (Proof Key for Code Exchange)** is essential for security, especially for public clients.

```
┌─────────┐              ┌──────────────┐              ┌─────────┐
│ User    │              │ LLM-CostOps  │              │   IdP   │
│ Browser │              │   (Client)   │              │ (AuthZ) │
└────┬────┘              └──────┬───────┘              └────┬────┘
     │                          │                           │
     │  1. Initiate Login       │                           │
     ├─────────────────────────>│                           │
     │                          │  Generate code_verifier   │
     │                          │  code_challenge = SHA256( │
     │                          │    code_verifier)         │
     │                          │                           │
     │  2. Redirect to AuthZ    │                           │
     │  + code_challenge        │                           │
     │<─────────────────────────┤                           │
     │                          │                           │
     │  3. Authorize + Consent  │                           │
     ├──────────────────────────┼──────────────────────────>│
     │                          │                           │
     │  4. Authorization Code   │                           │
     │<─────────────────────────┼───────────────────────────┤
     │                          │                           │
     │  5. Forward Auth Code    │                           │
     ├─────────────────────────>│                           │
     │                          │                           │
     │                          │  6. Exchange Code for Token
     │                          │  POST /token              │
     │                          │  + code                   │
     │                          │  + code_verifier          │
     │                          ├──────────────────────────>│
     │                          │                           │
     │                          │  7. Access + ID Tokens    │
     │                          │<──────────────────────────┤
     │                          │                           │
     │  8. Session Created      │                           │
     │<─────────────────────────┤                           │
```

#### PKCE Implementation

```rust
// Pseudo-code for PKCE implementation
use base64::URL_SAFE_NO_PAD;
use sha2::{Sha256, Digest};
use rand::Rng;

// Generate code verifier (43-128 characters)
fn generate_code_verifier() -> String {
    let random_bytes: Vec<u8> = (0..32)
        .map(|_| rand::thread_rng().gen::<u8>())
        .collect();
    base64::encode_config(&random_bytes, URL_SAFE_NO_PAD)
}

// Generate code challenge (SHA256 of verifier)
fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    base64::encode_config(&hash, URL_SAFE_NO_PAD)
}

// Authorization request parameters
struct AuthorizationRequest {
    response_type: "code",
    client_id: "llm-cost-ops",
    redirect_uri: "https://llm-cost-ops.example.com/oauth/callback",
    scope: "openid profile email",
    state: String,  // CSRF token
    code_challenge: String,
    code_challenge_method: "S256",
    nonce: String,  // For ID token validation
}
```

#### OpenID Connect ID Token

```json
{
  "iss": "https://accounts.google.com",
  "sub": "110169484474386276334",
  "azp": "llm-cost-ops-client-id",
  "aud": "llm-cost-ops-client-id",
  "iat": 1731672000,
  "exp": 1731675600,
  "nonce": "abc123xyz",

  "email": "user@example.com",
  "email_verified": true,
  "name": "John Doe",
  "given_name": "John",
  "family_name": "Doe",
  "picture": "https://lh3.googleusercontent.com/a/...",
  "locale": "en",

  "hd": "example.com",
  "groups": ["org_admin", "billing"]
}
```

#### ID Token Validation Requirements

**MUST Validate:**
1. **Signature**: Verify JWT signature using IdP's public key (from JWKS endpoint)
2. **Issuer (iss)**: Match expected IdP issuer URL
3. **Audience (aud)**: Match client_id
4. **Expiration (exp)**: Token not expired
5. **Issued At (iat)**: Token not issued in future
6. **Nonce**: Matches nonce from auth request (replay prevention)

```rust
// Pseudo-code for ID token validation
async fn validate_id_token(
    token: &str,
    expected_issuer: &str,
    expected_audience: &str,
    expected_nonce: &str,
) -> Result<Claims, TokenError> {
    // 1. Decode header to get key ID (kid)
    let header = decode_header(token)?;

    // 2. Fetch JWKS from IdP
    let jwks = fetch_jwks(&format!("{}/.well-known/jwks.json", expected_issuer)).await?;

    // 3. Find matching key
    let key = jwks.find_key(&header.kid)?;

    // 4. Verify signature
    let claims = verify_signature(token, key)?;

    // 5. Validate claims
    if claims.iss != expected_issuer {
        return Err(TokenError::InvalidIssuer);
    }
    if claims.aud != expected_audience {
        return Err(TokenError::InvalidAudience);
    }
    if claims.exp < current_timestamp() {
        return Err(TokenError::Expired);
    }
    if claims.nonce != expected_nonce {
        return Err(TokenError::InvalidNonce);
    }

    Ok(claims)
}
```

#### UserInfo Endpoint

After obtaining access token, retrieve additional user information:

```http
GET /userinfo HTTP/1.1
Host: accounts.google.com
Authorization: Bearer ya29.a0AfH6SMBx...

HTTP/1.1 200 OK
Content-Type: application/json

{
  "sub": "110169484474386276334",
  "email": "user@example.com",
  "email_verified": true,
  "name": "John Doe",
  "given_name": "John",
  "family_name": "Doe",
  "picture": "https://...",
  "locale": "en"
}
```

---

### 1.3 SCIM 2.0 (System for Cross-domain Identity Management)

#### Overview
SCIM 2.0 standardizes user provisioning and deprovisioning, enabling automated user lifecycle management.

#### Core Resources

**User Resource Schema**
```json
{
  "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
  "id": "2819c223-7f76-453a-919d-413861904646",
  "userName": "user@example.com",
  "externalId": "employee-12345",
  "name": {
    "formatted": "John Doe",
    "familyName": "Doe",
    "givenName": "John"
  },
  "emails": [
    {
      "value": "user@example.com",
      "type": "work",
      "primary": true
    }
  ],
  "active": true,
  "groups": [
    {
      "value": "group-id-123",
      "display": "Engineering"
    }
  ],
  "meta": {
    "resourceType": "User",
    "created": "2025-11-15T10:00:00Z",
    "lastModified": "2025-11-15T10:00:00Z",
    "version": "W/\"abc123\""
  }
}
```

**Group Resource Schema**
```json
{
  "schemas": ["urn:ietf:params:scim:schemas:core:2.0:Group"],
  "id": "group-id-123",
  "displayName": "Engineering",
  "members": [
    {
      "value": "2819c223-7f76-453a-919d-413861904646",
      "display": "John Doe",
      "type": "User"
    }
  ],
  "meta": {
    "resourceType": "Group",
    "created": "2025-11-15T10:00:00Z"
  }
}
```

#### SCIM Operations

**Create User**
```http
POST /scim/v2/Users HTTP/1.1
Host: llm-cost-ops.example.com
Authorization: Bearer {token}
Content-Type: application/scim+json

{
  "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
  "userName": "newuser@example.com",
  "name": {
    "givenName": "Jane",
    "familyName": "Smith"
  },
  "emails": [{"value": "newuser@example.com", "primary": true}],
  "active": true
}
```

**Update User (PATCH)**
```http
PATCH /scim/v2/Users/2819c223 HTTP/1.1
Authorization: Bearer {token}
Content-Type: application/scim+json

{
  "schemas": ["urn:ietf:params:scim:api:messages:2.0:PatchOp"],
  "Operations": [
    {
      "op": "replace",
      "path": "active",
      "value": false
    }
  ]
}
```

**Delete User (Deprovisioning)**
```http
DELETE /scim/v2/Users/2819c223 HTTP/1.1
Authorization: Bearer {token}
```

**Query Users (Filtering)**
```http
GET /scim/v2/Users?filter=emails[type eq "work" and value co "@example.com"]&startIndex=1&count=10
Authorization: Bearer {token}
```

#### SCIM Implementation Requirements

**Endpoints Required:**
- `GET /scim/v2/Users` - List users
- `GET /scim/v2/Users/{id}` - Get user
- `POST /scim/v2/Users` - Create user
- `PUT /scim/v2/Users/{id}` - Replace user
- `PATCH /scim/v2/Users/{id}` - Update user
- `DELETE /scim/v2/Users/{id}` - Delete user
- `GET /scim/v2/Groups` - List groups
- `POST /scim/v2/Groups` - Create group

**Error Handling:**
```json
{
  "schemas": ["urn:ietf:params:scim:api:messages:2.0:Error"],
  "status": "409",
  "scimType": "uniqueness",
  "detail": "User with userName 'user@example.com' already exists"
}
```

---

## 2. Identity Provider Integration

### 2.1 Okta Integration

#### Overview
Okta is a leading enterprise IdP supporting both SAML 2.0 and OpenID Connect.

#### SAML 2.0 Configuration

**Okta Setup:**
1. Create SAML 2.0 Application in Okta Admin Console
2. Configure Single Sign-On URL: `https://llm-cost-ops.example.com/saml/acs`
3. Configure Audience URI: `llm-cost-ops`
4. Add Attribute Statements:
   - `email` → `user.email`
   - `firstName` → `user.firstName`
   - `lastName` → `user.lastName`
   - `groups` → `appuser.groups` (array)

**Download Metadata:**
```
https://your-tenant.okta.com/app/{app-id}/sso/saml/metadata
```

**Attribute Mapping:**
```
Okta Attribute      →  LLM-CostOps Field
─────────────────────────────────────────
user.email          →  email (NameID)
user.firstName      →  first_name
user.lastName       →  last_name
user.login          →  username
appuser.groups      →  roles[] (mapped to RBAC)
user.department     →  department
user.organization   →  organization_id
```

#### OpenID Connect Configuration

**OAuth 2.0 Settings:**
- **Authorization Server:** `https://your-tenant.okta.com/oauth2/default`
- **Client ID:** Provided by Okta
- **Client Secret:** Provided by Okta
- **Scopes:** `openid profile email groups`
- **Redirect URI:** `https://llm-cost-ops.example.com/oauth/callback`

**Token Endpoint:**
```
POST https://your-tenant.okta.com/oauth2/default/v1/token
```

**Discovery Document:**
```
https://your-tenant.okta.com/oauth2/default/.well-known/openid-configuration
```

**Group Claims:**
Configure custom claims in Authorization Server:
```json
{
  "name": "groups",
  "type": "EXPRESSION",
  "value": "Arrays.flatten(isMemberOfGroupNameStartsWith(\"LLM-CostOps\", 50))"
}
```

#### SCIM Provisioning

**Okta SCIM Configuration:**
- **SCIM Base URL:** `https://llm-cost-ops.example.com/scim/v2`
- **Unique Identifier:** `userName`
- **Supported Provisioning:** Create, Update, Deactivate
- **Authentication:** OAuth 2.0 Bearer Token

**Lifecycle Actions:**
- User assigned to app → Create user (JIT)
- User updated in Okta → Update user
- User unassigned → Deactivate user
- Group membership changed → Update user groups

---

### 2.2 Auth0 Integration

#### Overview
Auth0 provides Universal Login with support for multiple identity sources and social providers.

#### OpenID Connect Setup

**Auth0 Application Configuration:**
- **Application Type:** Regular Web Application
- **Token Endpoint Authentication:** POST
- **Allowed Callback URLs:** `https://llm-cost-ops.example.com/oauth/callback`
- **Allowed Logout URLs:** `https://llm-cost-ops.example.com/logout`
- **Allowed Web Origins:** `https://llm-cost-ops.example.com`

**Connection Settings:**
```json
{
  "domain": "your-tenant.us.auth0.com",
  "clientID": "your-client-id",
  "clientSecret": "your-client-secret",
  "audience": "https://llm-cost-ops.example.com",
  "scope": "openid profile email offline_access"
}
```

**Custom Claims (Rules):**
```javascript
// Auth0 Rule: Add custom claims
function addCustomClaims(user, context, callback) {
  const namespace = 'https://llm-cost-ops.example.com/';

  context.idToken[namespace + 'organization_id'] = user.org_id;
  context.idToken[namespace + 'roles'] = user.app_metadata.roles || [];
  context.idToken[namespace + 'permissions'] = user.app_metadata.permissions || [];

  callback(null, user, context);
}
```

#### Management API Integration

**Retrieve User Profile:**
```http
GET https://your-tenant.us.auth0.com/api/v2/users/{user_id}
Authorization: Bearer {management_api_token}

{
  "user_id": "auth0|123456",
  "email": "user@example.com",
  "email_verified": true,
  "name": "John Doe",
  "app_metadata": {
    "organization_id": "org-123",
    "roles": ["org_admin", "billing"]
  },
  "user_metadata": {
    "department": "Engineering"
  }
}
```

**Update User Metadata:**
```http
PATCH https://your-tenant.us.auth0.com/api/v2/users/{user_id}
Authorization: Bearer {management_api_token}
Content-Type: application/json

{
  "app_metadata": {
    "roles": ["org_admin", "billing", "auditor"]
  }
}
```

---

### 2.3 Azure AD / Entra ID Integration

#### Overview
Microsoft Azure AD (now Entra ID) is the de facto standard for organizations using Microsoft 365.

#### SAML 2.0 Enterprise Application

**Azure AD Setup:**
1. Create Enterprise Application
2. Select SAML-based Sign-on
3. Configure:
   - **Identifier (Entity ID):** `llm-cost-ops`
   - **Reply URL (ACS):** `https://llm-cost-ops.example.com/saml/acs`
   - **Sign on URL:** `https://llm-cost-ops.example.com`

**Claims Mapping:**
```
Azure AD Claim                    →  SAML Attribute
────────────────────────────────────────────────────
user.userprincipalname            →  NameID
user.mail                         →  email
user.givenname                    →  firstName
user.surname                      →  lastName
user.groups                       →  groups (requires App Role assignment)
user.department                   →  department
user.companyname                  →  organization
```

**Group Claims:**
Configure group claims in Token Configuration:
- Emit groups as: **sAMAccountName**
- Groups assigned to application: **Yes**

#### OAuth 2.0 / OpenID Connect

**App Registration:**
- **Application (client) ID:** From Azure AD
- **Directory (tenant) ID:** Your Azure AD tenant
- **Client Secret:** Generate in Certificates & Secrets
- **Redirect URI:** `https://llm-cost-ops.example.com/oauth/callback`

**Microsoft Identity Platform Endpoints:**
```
Authorization:
https://login.microsoftonline.com/{tenant}/oauth2/v2.0/authorize

Token:
https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token

JWKS:
https://login.microsoftonline.com/{tenant}/discovery/v2.0/keys
```

**Scopes:**
- `openid` - ID token
- `profile` - Name, username
- `email` - Email address
- `offline_access` - Refresh token
- `User.Read` - Microsoft Graph profile

**ID Token Example:**
```json
{
  "iss": "https://login.microsoftonline.com/{tenant}/v2.0",
  "sub": "AAAAAAAAAAAAAAAAAAAAAIkzqFV...",
  "aud": "6e74172b-be56-4843-9ff4-e66a39bb12e3",
  "exp": 1731675600,
  "iat": 1731672000,
  "nbf": 1731672000,

  "email": "user@contoso.com",
  "name": "John Doe",
  "preferred_username": "user@contoso.com",
  "oid": "00000000-0000-0000-66f3-3332eca7ea81",
  "tid": "{tenant-id}",

  "roles": ["Admin", "Billing"],
  "groups": ["group-id-1", "group-id-2"]
}
```

#### Microsoft Graph API Integration

**Retrieve User Profile:**
```http
GET https://graph.microsoft.com/v1.0/me
Authorization: Bearer {access_token}

{
  "id": "87d349ed-44d7-43e1-9a83-5f2406dee5bd",
  "displayName": "John Doe",
  "mail": "user@contoso.com",
  "userPrincipalName": "user@contoso.com",
  "jobTitle": "Senior Engineer",
  "department": "Engineering",
  "officeLocation": "San Francisco"
}
```

**Retrieve Group Memberships:**
```http
GET https://graph.microsoft.com/v1.0/me/memberOf
Authorization: Bearer {access_token}

{
  "value": [
    {
      "@odata.type": "#microsoft.graph.group",
      "id": "group-id-1",
      "displayName": "LLM-CostOps Admins"
    }
  ]
}
```

---

### 2.4 Google Workspace Integration

#### Overview
Google Workspace (formerly G Suite) uses OAuth 2.0 and OpenID Connect for authentication.

#### OAuth 2.0 Configuration

**Google Cloud Console Setup:**
1. Create OAuth 2.0 Client ID
2. Application type: **Web application**
3. Authorized redirect URIs: `https://llm-cost-ops.example.com/oauth/callback`
4. Authorized JavaScript origins: `https://llm-cost-ops.example.com`

**OAuth Endpoints:**
```
Authorization:
https://accounts.google.com/o/oauth2/v2/auth

Token:
https://oauth2.googleapis.com/token

JWKS:
https://www.googleapis.com/oauth2/v3/certs

UserInfo:
https://openidconnect.googleapis.com/v1/userinfo
```

**Scopes:**
- `openid` - Required for OIDC
- `email` - Email address
- `profile` - Name, picture
- `https://www.googleapis.com/auth/admin.directory.user.readonly` - Directory API (optional)

**ID Token Claims:**
```json
{
  "iss": "https://accounts.google.com",
  "sub": "110169484474386276334",
  "azp": "your-client-id.apps.googleusercontent.com",
  "aud": "your-client-id.apps.googleusercontent.com",
  "iat": 1731672000,
  "exp": 1731675600,

  "email": "user@example.com",
  "email_verified": true,
  "name": "John Doe",
  "given_name": "John",
  "family_name": "Doe",
  "picture": "https://lh3.googleusercontent.com/...",
  "locale": "en",

  "hd": "example.com"  // Hosted domain (for Workspace)
}
```

#### Domain Restriction

**Validate Hosted Domain:**
```rust
// Only allow users from specific Google Workspace domain
fn validate_workspace_domain(id_token: &IdToken) -> Result<(), AuthError> {
    match id_token.hd.as_ref() {
        Some(hd) if hd == "example.com" => Ok(()),
        Some(hd) => Err(AuthError::InvalidDomain(hd.clone())),
        None => Err(AuthError::MissingDomain),
    }
}
```

#### Google Directory API (Optional)

**Retrieve Organization Units:**
```http
GET https://admin.googleapis.com/admin/directory/v1/users/{userKey}
Authorization: Bearer {access_token}

{
  "primaryEmail": "user@example.com",
  "name": {
    "fullName": "John Doe",
    "givenName": "John",
    "familyName": "Doe"
  },
  "orgUnitPath": "/Engineering",
  "customSchemas": {
    "CustomAttributes": {
      "EmployeeID": "12345",
      "CostCenter": "ENG-001"
    }
  }
}
```

---

### 2.5 GitHub Enterprise Integration

#### Overview
GitHub provides OAuth 2.0 for authentication, commonly used for developer-focused applications.

#### OAuth 2.0 Setup

**GitHub OAuth App Registration:**
1. Settings → Developer settings → OAuth Apps
2. Application name: **LLM-CostOps**
3. Homepage URL: `https://llm-cost-ops.example.com`
4. Authorization callback URL: `https://llm-cost-ops.example.com/oauth/callback`

**OAuth Endpoints:**
```
Authorization:
https://github.com/login/oauth/authorize

Token:
https://github.com/login/oauth/access_token

User API:
https://api.github.com/user
```

**Scopes:**
- `read:user` - Read user profile
- `user:email` - Access email addresses
- `read:org` - Read organization membership

**Authorization Request:**
```http
GET https://github.com/login/oauth/authorize?
  client_id=your-client-id&
  redirect_uri=https://llm-cost-ops.example.com/oauth/callback&
  scope=read:user user:email read:org&
  state=random-state-token
```

**Token Exchange:**
```http
POST https://github.com/login/oauth/access_token
Content-Type: application/json
Accept: application/json

{
  "client_id": "your-client-id",
  "client_secret": "your-client-secret",
  "code": "authorization-code",
  "redirect_uri": "https://llm-cost-ops.example.com/oauth/callback"
}

Response:
{
  "access_token": "gho_...",
  "token_type": "bearer",
  "scope": "read:user,user:email,read:org"
}
```

#### User Profile Retrieval

**Get Authenticated User:**
```http
GET https://api.github.com/user
Authorization: Bearer {access_token}

{
  "login": "johndoe",
  "id": 1234567,
  "email": "user@example.com",
  "name": "John Doe",
  "company": "Example Corp",
  "location": "San Francisco",
  "created_at": "2015-01-01T00:00:00Z"
}
```

**Get Organization Membership:**
```http
GET https://api.github.com/user/orgs
Authorization: Bearer {access_token}

[
  {
    "login": "example-org",
    "id": 7654321,
    "url": "https://api.github.com/orgs/example-org"
  }
]
```

**Attribute Mapping:**
```
GitHub Field    →  LLM-CostOps Field
────────────────────────────────────
login           →  username
email           →  email
name            →  full_name
company         →  organization
orgs[].login    →  organizations[]
```

---

## 3. Security Requirements

### 3.1 CSRF Protection for Auth Flows

#### State Parameter Validation

**Purpose:** Prevent Cross-Site Request Forgery attacks during OAuth/OIDC flows.

**Implementation:**
```rust
use rand::Rng;
use sha2::{Sha256, Digest};

// Generate cryptographically secure state token
fn generate_state_token() -> String {
    let random_bytes: Vec<u8> = (0..32)
        .map(|_| rand::thread_rng().gen::<u8>())
        .collect();

    let mut hasher = Sha256::new();
    hasher.update(&random_bytes);
    let hash = hasher.finalize();

    base64::encode_config(&hash, base64::URL_SAFE_NO_PAD)
}

// Store state in session with expiration
async fn store_state(
    session: &Session,
    state: String,
    expires_in: Duration,
) -> Result<(), SessionError> {
    session.insert("oauth_state", state)?;
    session.insert("oauth_state_exp", Utc::now() + expires_in)?;
    Ok(())
}

// Validate state from callback
async fn validate_state(
    session: &Session,
    received_state: &str,
) -> Result<(), AuthError> {
    let stored_state: Option<String> = session.get("oauth_state")?;
    let exp: Option<DateTime<Utc>> = session.get("oauth_state_exp")?;

    // Check state exists
    let stored = stored_state.ok_or(AuthError::MissingState)?;

    // Check not expired
    let expiration = exp.ok_or(AuthError::MissingState)?;
    if Utc::now() > expiration {
        return Err(AuthError::StateExpired);
    }

    // Constant-time comparison to prevent timing attacks
    if !constant_time_eq(stored.as_bytes(), received_state.as_bytes()) {
        return Err(AuthError::InvalidState);
    }

    // Clear state after validation
    session.remove("oauth_state")?;
    session.remove("oauth_state_exp")?;

    Ok(())
}
```

**Requirements:**
- State token MUST be at least 128 bits of entropy
- MUST be stored server-side (not in URL or client storage)
- MUST expire after 10 minutes
- MUST be removed after one-time use
- MUST use constant-time comparison to prevent timing attacks

---

### 3.2 Token Validation

#### JWT Signature Verification

**Algorithm Support:**
- **Symmetric (HMAC):** HS256, HS384, HS512 (for internal tokens)
- **Asymmetric (RSA):** RS256, RS384, RS512 (for IdP tokens)
- **Asymmetric (ECDSA):** ES256, ES384, ES512 (recommended)

**Key Management:**
```rust
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct JwksKey {
    kty: String,        // Key type (RSA, EC)
    kid: String,        // Key ID
    use_: String,       // Key use (sig)
    alg: String,        // Algorithm
    n: Option<String>,  // RSA modulus
    e: Option<String>,  // RSA exponent
    x: Option<String>,  // EC x coordinate
    y: Option<String>,  // EC y coordinate
}

#[derive(Debug, Serialize, Deserialize)]
struct Jwks {
    keys: Vec<JwksKey>,
}

// Fetch JWKS from IdP
async fn fetch_jwks(jwks_uri: &str) -> Result<Jwks, JwksError> {
    let response = reqwest::get(jwks_uri)
        .await?
        .json::<Jwks>()
        .await?;

    // Cache JWKS with TTL (e.g., 1 hour)
    cache_jwks(&response, Duration::hours(1)).await?;

    Ok(response)
}

// Verify token signature
async fn verify_token_signature(
    token: &str,
    jwks_uri: &str,
) -> Result<Claims, TokenError> {
    // Decode header to get kid
    let header = jsonwebtoken::decode_header(token)?;
    let kid = header.kid.ok_or(TokenError::MissingKid)?;

    // Fetch JWKS (from cache or IdP)
    let jwks = get_cached_jwks_or_fetch(jwks_uri).await?;

    // Find matching key
    let jwk = jwks.keys.iter()
        .find(|k| k.kid == kid)
        .ok_or(TokenError::KeyNotFound)?;

    // Convert JWK to DecodingKey
    let decoding_key = match jwk.kty.as_str() {
        "RSA" => {
            let n = jwk.n.as_ref().ok_or(TokenError::InvalidKey)?;
            let e = jwk.e.as_ref().ok_or(TokenError::InvalidKey)?;
            DecodingKey::from_rsa_components(n, e)?
        }
        "EC" => {
            // EC key handling
            todo!("EC key support")
        }
        _ => return Err(TokenError::UnsupportedKeyType),
    };

    // Verify signature
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = false; // Validate separately

    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;

    Ok(token_data.claims)
}
```

#### Expiration Validation

```rust
fn validate_token_expiration(claims: &Claims) -> Result<(), TokenError> {
    let now = Utc::now().timestamp();

    // Check expiration (exp)
    if claims.exp < now {
        return Err(TokenError::Expired);
    }

    // Check not before (nbf)
    if let Some(nbf) = claims.nbf {
        if nbf > now {
            return Err(TokenError::NotYetValid);
        }
    }

    // Check issued at (iat) - reject if issued in future
    if let Some(iat) = claims.iat {
        // Allow 60 seconds clock skew
        if iat > now + 60 {
            return Err(TokenError::InvalidIssuedAt);
        }
    }

    Ok(())
}
```

---

### 3.3 Redirect URI Validation

**Purpose:** Prevent open redirect vulnerabilities and authorization code interception.

**Validation Rules:**
1. Exact match only (no pattern matching)
2. HTTPS required for production
3. No wildcards allowed
4. Validate scheme, host, port, and path
5. Maintain whitelist of allowed URIs

```rust
use url::Url;

struct RedirectUriValidator {
    allowed_uris: Vec<String>,
    allow_localhost: bool, // For development only
}

impl RedirectUriValidator {
    fn validate(&self, redirect_uri: &str) -> Result<(), ValidationError> {
        let url = Url::parse(redirect_uri)
            .map_err(|_| ValidationError::InvalidUri)?;

        // Check scheme (HTTPS in production)
        if !self.allow_localhost && url.scheme() != "https" {
            return Err(ValidationError::InsecureScheme);
        }

        // Check for localhost in production
        if !self.allow_localhost && self.is_localhost(&url) {
            return Err(ValidationError::LocalhostNotAllowed);
        }

        // Exact match against whitelist
        if !self.allowed_uris.contains(&redirect_uri.to_string()) {
            return Err(ValidationError::UriNotWhitelisted);
        }

        Ok(())
    }

    fn is_localhost(&self, url: &Url) -> bool {
        match url.host_str() {
            Some("localhost") | Some("127.0.0.1") | Some("::1") => true,
            _ => false,
        }
    }
}
```

---

### 3.4 Replay Attack Prevention

#### SAML Assertion ID Tracking

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashSet;
use chrono::{DateTime, Duration, Utc};

struct AssertionIdStore {
    used_ids: Arc<RwLock<HashSet<String>>>,
    expiry_duration: Duration,
}

impl AssertionIdStore {
    fn new() -> Self {
        Self {
            used_ids: Arc::new(RwLock::new(HashSet::new())),
            expiry_duration: Duration::minutes(5),
        }
    }

    async fn mark_used(&self, assertion_id: &str) -> Result<(), ReplayError> {
        let mut ids = self.used_ids.write().await;

        // Check if already used
        if ids.contains(assertion_id) {
            return Err(ReplayError::AssertionReplayed);
        }

        // Mark as used
        ids.insert(assertion_id.to_string());

        // Schedule cleanup (in production, use Redis with TTL)
        let id = assertion_id.to_string();
        let store = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(store.expiry_duration.to_std().unwrap()).await;
            let mut ids = store.used_ids.write().await;
            ids.remove(&id);
        });

        Ok(())
    }
}
```

#### OAuth Nonce Validation

```rust
async fn validate_oidc_nonce(
    id_token: &IdToken,
    session: &Session,
) -> Result<(), TokenError> {
    // Retrieve expected nonce from session
    let expected_nonce: Option<String> = session.get("oidc_nonce")?;
    let stored_nonce = expected_nonce.ok_or(TokenError::MissingNonce)?;

    // Compare with ID token nonce
    let token_nonce = id_token.nonce.as_ref()
        .ok_or(TokenError::MissingNonce)?;

    if !constant_time_eq(stored_nonce.as_bytes(), token_nonce.as_bytes()) {
        return Err(TokenError::NonceMismatch);
    }

    // Clear nonce after one-time use
    session.remove("oidc_nonce")?;

    Ok(())
}
```

---

## 4. Enterprise Features

### 4.1 Just-in-Time (JIT) User Provisioning

#### Overview
JIT provisioning automatically creates user accounts upon first SSO login, eliminating manual user creation.

#### JIT Provisioning Flow

```
┌─────────┐            ┌──────────────┐            ┌─────────┐
│  User   │            │ LLM-CostOps  │            │   IdP   │
└────┬────┘            └──────┬───────┘            └────┬────┘
     │                        │                         │
     │  1. SSO Login          │                         │
     ├───────────────────────>│                         │
     │                        │  2. Authenticate        │
     │                        ├────────────────────────>│
     │                        │                         │
     │                        │  3. SAML/OIDC Response  │
     │                        │<────────────────────────┤
     │                        │                         │
     │                        │  4. Check if user exists│
     │                        │  - Query by email/NameID│
     │                        │                         │
     │                        │  5. User NOT found      │
     │                        │  → Create new user      │
     │                        │  → Map attributes       │
     │                        │  → Assign default role  │
     │                        │                         │
     │  6. Login Success      │                         │
     │<───────────────────────┤                         │
```

#### Implementation

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct JitProvisioningConfig {
    enabled: bool,
    default_role: String,
    attribute_mapping: HashMap<String, String>,
    require_email_verification: bool,
    auto_activate: bool,
}

#[derive(Debug)]
struct SamlAttributes {
    email: String,
    first_name: Option<String>,
    last_name: Option<String>,
    groups: Vec<String>,
    department: Option<String>,
    organization_id: Option<String>,
}

async fn jit_provision_user(
    saml_attributes: SamlAttributes,
    config: &JitProvisioningConfig,
    user_repo: &dyn UserRepository,
    rbac_manager: &RbacManager,
) -> Result<User, ProvisioningError> {
    // Check if user already exists
    if let Some(existing_user) = user_repo.find_by_email(&saml_attributes.email).await? {
        // Update user attributes if changed
        return update_user_from_saml(existing_user, saml_attributes, user_repo).await;
    }

    // Create new user
    let user = User {
        id: uuid::Uuid::new_v4().to_string(),
        email: saml_attributes.email.clone(),
        first_name: saml_attributes.first_name,
        last_name: saml_attributes.last_name,
        organization_id: saml_attributes.organization_id
            .unwrap_or_else(|| "default-org".to_string()),
        department: saml_attributes.department,
        active: config.auto_activate,
        email_verified: !config.require_email_verification,
        sso_provider: Some("saml".to_string()),
        sso_subject: Some(saml_attributes.email.clone()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Save user
    user_repo.create(user.clone()).await?;

    // Assign default role
    rbac_manager.assign_user_role(&user.id, &config.default_role).await?;

    // Map SAML groups to roles
    for group in saml_attributes.groups {
        if let Some(role_id) = map_group_to_role(&group) {
            rbac_manager.assign_user_role(&user.id, &role_id).await?;
        }
    }

    // Audit log
    audit_log(AuditEvent {
        event_type: AuditEventType::UserProvisioned,
        user_id: Some(user.id.clone()),
        details: format!("JIT provisioned via SAML: {}", user.email),
        timestamp: Utc::now(),
    }).await?;

    Ok(user)
}

fn map_group_to_role(group: &str) -> Option<String> {
    match group {
        "LLM-CostOps-Admins" => Some("org_admin".to_string()),
        "LLM-CostOps-Billing" => Some("billing".to_string()),
        "LLM-CostOps-ReadOnly" => Some("read_only".to_string()),
        "LLM-CostOps-Auditors" => Some("auditor".to_string()),
        _ => None,
    }
}
```

---

### 4.2 Attribute Mapping

#### SAML Assertion to User Profile

```rust
struct AttributeMapper {
    mappings: HashMap<String, AttributeMapping>,
}

struct AttributeMapping {
    source_attribute: String,
    target_field: String,
    transform: Option<Box<dyn Fn(String) -> String>>,
    required: bool,
}

impl AttributeMapper {
    fn map_saml_attributes(
        &self,
        saml_assertion: &SamlAssertion,
    ) -> Result<UserProfile, MappingError> {
        let mut profile = UserProfile::default();

        for (key, mapping) in &self.mappings {
            let value = saml_assertion.attributes.get(&mapping.source_attribute);

            match value {
                Some(v) => {
                    let transformed = if let Some(transform) = &mapping.transform {
                        transform(v.clone())
                    } else {
                        v.clone()
                    };

                    self.set_field(&mut profile, &mapping.target_field, transformed)?;
                }
                None if mapping.required => {
                    return Err(MappingError::MissingRequiredAttribute(
                        mapping.source_attribute.clone()
                    ));
                }
                None => {
                    // Optional attribute, skip
                }
            }
        }

        Ok(profile)
    }

    fn set_field(
        &self,
        profile: &mut UserProfile,
        field: &str,
        value: String,
    ) -> Result<(), MappingError> {
        match field {
            "email" => profile.email = value,
            "first_name" => profile.first_name = Some(value),
            "last_name" => profile.last_name = Some(value),
            "organization_id" => profile.organization_id = value,
            "department" => profile.department = Some(value),
            _ => return Err(MappingError::UnknownField(field.to_string())),
        }
        Ok(())
    }
}

// Configuration example
fn create_attribute_mapper() -> AttributeMapper {
    let mut mappings = HashMap::new();

    mappings.insert("email".to_string(), AttributeMapping {
        source_attribute: "email".to_string(),
        target_field: "email".to_string(),
        transform: Some(Box::new(|v| v.to_lowercase())),
        required: true,
    });

    mappings.insert("firstName".to_string(), AttributeMapping {
        source_attribute: "firstName".to_string(),
        target_field: "first_name".to_string(),
        transform: None,
        required: false,
    });

    mappings.insert("organization".to_string(), AttributeMapping {
        source_attribute: "companyName".to_string(),
        target_field: "organization_id".to_string(),
        transform: Some(Box::new(|v| {
            // Convert company name to org ID
            format!("org-{}", v.to_lowercase().replace(" ", "-"))
        })),
        required: true,
    });

    AttributeMapper { mappings }
}
```

---

### 4.3 Group/Role Synchronization

#### Automatic Role Assignment

```rust
#[derive(Debug, Serialize, Deserialize)]
struct RoleSyncConfig {
    sync_enabled: bool,
    group_role_mappings: HashMap<String, Vec<String>>,
    remove_unmatched_roles: bool,
}

async fn sync_user_roles(
    user_id: &str,
    sso_groups: Vec<String>,
    config: &RoleSyncConfig,
    rbac_manager: &RbacManager,
) -> Result<(), SyncError> {
    if !config.sync_enabled {
        return Ok(());
    }

    // Determine roles from groups
    let mut desired_roles = HashSet::new();
    for group in &sso_groups {
        if let Some(roles) = config.group_role_mappings.get(group) {
            desired_roles.extend(roles.iter().cloned());
        }
    }

    // Get current roles
    let current_roles = rbac_manager.get_user_roles(user_id).await;
    let current_role_ids: HashSet<String> = current_roles
        .iter()
        .map(|r| r.id.clone())
        .collect();

    // Add missing roles
    for role_id in desired_roles.difference(&current_role_ids) {
        rbac_manager.assign_user_role(user_id, role_id).await?;
    }

    // Remove unmatched roles (if configured)
    if config.remove_unmatched_roles {
        for role_id in current_role_ids.difference(&desired_roles) {
            // Don't remove system roles or manually assigned roles
            if !is_system_role(role_id) && is_sso_synced_role(role_id) {
                rbac_manager.remove_user_role(user_id, role_id).await?;
            }
        }
    }

    Ok(())
}
```

---

### 4.4 Multi-Factor Authentication (MFA) Support

#### MFA Enforcement

```rust
#[derive(Debug, Serialize, Deserialize)]
struct MfaConfig {
    required: bool,
    required_for_roles: Vec<String>,
    trusted_idps: Vec<String>, // IdPs that handle MFA
}

fn validate_mfa_requirement(
    claims: &JwtClaims,
    user_roles: &[Role],
    config: &MfaConfig,
) -> Result<(), MfaError> {
    if !config.required {
        return Ok(());
    }

    // Check if user has roles requiring MFA
    let requires_mfa = user_roles.iter().any(|r| {
        config.required_for_roles.contains(&r.id)
    });

    if !requires_mfa {
        return Ok(());
    }

    // Check SAML AuthnContext or OIDC amr claim
    let mfa_completed = check_mfa_claims(claims)?;

    if !mfa_completed {
        return Err(MfaError::MfaRequired);
    }

    Ok(())
}

fn check_mfa_claims(claims: &JwtClaims) -> Result<bool, MfaError> {
    // For OIDC, check 'amr' (Authentication Methods Reference)
    if let Some(amr) = claims.amr.as_ref() {
        let has_mfa = amr.iter().any(|method| {
            matches!(method.as_str(), "mfa" | "otp" | "sms" | "totp")
        });
        return Ok(has_mfa);
    }

    // For SAML, check AuthnContextClassRef
    if let Some(acr) = claims.acr.as_ref() {
        let has_mfa = acr.contains("MultiFactor");
        return Ok(has_mfa);
    }

    Ok(false)
}
```

---

### 4.5 Session Lifetime Management

#### Adaptive Session Timeouts

```rust
#[derive(Debug, Serialize, Deserialize)]
struct SessionConfig {
    default_lifetime_secs: i64,
    max_lifetime_secs: i64,
    idle_timeout_secs: i64,
    remember_me_lifetime_secs: i64,
    role_based_lifetimes: HashMap<String, i64>,
}

struct SessionManager {
    config: SessionConfig,
}

impl SessionManager {
    fn calculate_session_lifetime(
        &self,
        user_roles: &[Role],
        remember_me: bool,
    ) -> Duration {
        if remember_me {
            return Duration::seconds(self.config.remember_me_lifetime_secs);
        }

        // Find shortest lifetime for user's roles
        let role_lifetime = user_roles.iter()
            .filter_map(|role| self.config.role_based_lifetimes.get(&role.id))
            .min()
            .copied();

        let lifetime_secs = role_lifetime
            .unwrap_or(self.config.default_lifetime_secs)
            .min(self.config.max_lifetime_secs);

        Duration::seconds(lifetime_secs)
    }

    async fn refresh_session(
        &self,
        session: &mut Session,
    ) -> Result<(), SessionError> {
        let last_activity: DateTime<Utc> = session.get("last_activity")?
            .unwrap_or_else(Utc::now);

        let idle_duration = Utc::now() - last_activity;

        // Check idle timeout
        if idle_duration.num_seconds() > self.config.idle_timeout_secs {
            return Err(SessionError::IdleTimeout);
        }

        // Update last activity
        session.insert("last_activity", Utc::now())?;

        Ok(())
    }
}
```

---

## 5. User Provisioning Strategies

### 5.1 Just-in-Time (JIT) vs SCIM Provisioning

#### Comparison Matrix

| Feature | JIT Provisioning | SCIM Provisioning |
|---------|------------------|-------------------|
| **Timing** | On first login | Real-time/scheduled |
| **Direction** | Pull (SP pulls from IdP) | Push (IdP pushes to SP) |
| **User Creation** | Automatic on SSO | Pre-created before login |
| **Deprovisioning** | Manual or periodic cleanup | Automatic on user removal |
| **Group Sync** | On each login | Real-time updates |
| **Complexity** | Low | Medium-High |
| **IdP Support** | Universal (SAML/OIDC) | Requires SCIM support |
| **Use Case** | Small-medium orgs | Enterprise with frequent changes |

#### Hybrid Approach (Recommended)

```rust
#[derive(Debug, Serialize, Deserialize)]
enum ProvisioningStrategy {
    JitOnly,
    ScimOnly,
    Hybrid {
        prefer_scim: bool,
        jit_fallback: bool,
    },
}

async fn provision_user(
    sso_subject: &str,
    attributes: SsoAttributes,
    strategy: &ProvisioningStrategy,
    user_repo: &dyn UserRepository,
) -> Result<User, ProvisioningError> {
    match strategy {
        ProvisioningStrategy::JitOnly => {
            jit_provision_user(attributes, user_repo).await
        }

        ProvisioningStrategy::ScimOnly => {
            // User must already exist via SCIM
            user_repo.find_by_sso_subject(sso_subject)
                .await?
                .ok_or(ProvisioningError::UserNotProvisioned)
        }

        ProvisioningStrategy::Hybrid { prefer_scim, jit_fallback } => {
            // Try SCIM-provisioned user first
            if let Some(user) = user_repo.find_by_sso_subject(sso_subject).await? {
                return Ok(user);
            }

            // Fall back to JIT if enabled
            if *jit_fallback {
                jit_provision_user(attributes, user_repo).await
            } else {
                Err(ProvisioningError::UserNotProvisioned)
            }
        }
    }
}
```

---

### 5.2 User Deprovisioning

#### Automated Deprovisioning Flow

```rust
#[derive(Debug, Serialize, Deserialize)]
struct DeprovisioningConfig {
    strategy: DeprovisioningStrategy,
    grace_period_days: u32,
    archive_data: bool,
}

#[derive(Debug, Serialize, Deserialize)]
enum DeprovisioningStrategy {
    Deactivate,           // Mark user inactive
    SoftDelete,           // Logical delete
    HardDelete,           // Physical delete (GDPR)
    Archive,              // Move to archive
}

async fn deprovision_user(
    user_id: &str,
    config: &DeprovisioningConfig,
    user_repo: &dyn UserRepository,
) -> Result<(), DeprovisioningError> {
    let user = user_repo.find_by_id(user_id).await?
        .ok_or(DeprovisioningError::UserNotFound)?;

    match config.strategy {
        DeprovisioningStrategy::Deactivate => {
            // Mark user as inactive
            user_repo.update_status(user_id, UserStatus::Inactive).await?;

            // Revoke all sessions
            session_manager.revoke_all_sessions(user_id).await?;

            // Revoke API keys
            api_key_manager.revoke_all_keys(user_id).await?;
        }

        DeprovisioningStrategy::SoftDelete => {
            // Logical delete
            user_repo.soft_delete(user_id).await?;

            // Schedule hard delete after grace period
            schedule_hard_delete(user_id, config.grace_period_days).await?;
        }

        DeprovisioningStrategy::HardDelete => {
            // Archive data if configured
            if config.archive_data {
                archive_user_data(user_id).await?;
            }

            // Physical delete
            user_repo.hard_delete(user_id).await?;
        }

        DeprovisioningStrategy::Archive => {
            // Move to archive storage
            archive_user(user_id).await?;
        }
    }

    // Audit log
    audit_log(AuditEvent {
        event_type: AuditEventType::UserDeprovisioned,
        user_id: Some(user_id.to_string()),
        details: format!("Deprovisioned via {:?}", config.strategy),
        timestamp: Utc::now(),
    }).await?;

    Ok(())
}
```

---

## 6. Session Management

### 6.1 Session Storage Options

#### Redis-Based Sessions (Recommended)

```rust
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct SessionData {
    user_id: String,
    organization_id: String,
    roles: Vec<String>,
    permissions: Vec<String>,
    created_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,
    ip_address: String,
    user_agent: String,
}

struct RedisSessionStore {
    client: redis::Client,
    prefix: String,
    default_ttl: i64,
}

impl RedisSessionStore {
    async fn create_session(
        &self,
        session_id: &str,
        data: SessionData,
        ttl: Option<i64>,
    ) -> Result<(), SessionError> {
        let mut conn = self.client.get_async_connection().await?;

        let key = format!("{}:{}", self.prefix, session_id);
        let value = serde_json::to_string(&data)?;
        let ttl = ttl.unwrap_or(self.default_ttl);

        // Set with expiration
        conn.set_ex(key, value, ttl as usize).await?;

        Ok(())
    }

    async fn get_session(
        &self,
        session_id: &str,
    ) -> Result<Option<SessionData>, SessionError> {
        let mut conn = self.client.get_async_connection().await?;

        let key = format!("{}:{}", self.prefix, session_id);
        let value: Option<String> = conn.get(&key).await?;

        match value {
            Some(v) => {
                let data: SessionData = serde_json::from_str(&v)?;
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }

    async fn refresh_session(
        &self,
        session_id: &str,
        extend_by: i64,
    ) -> Result<(), SessionError> {
        let mut conn = self.client.get_async_connection().await?;

        let key = format!("{}:{}", self.prefix, session_id);
        conn.expire(&key, extend_by as usize).await?;

        Ok(())
    }

    async fn revoke_session(
        &self,
        session_id: &str,
    ) -> Result<(), SessionError> {
        let mut conn = self.client.get_async_connection().await?;

        let key = format!("{}:{}", self.prefix, session_id);
        conn.del(&key).await?;

        Ok(())
    }

    async fn revoke_all_user_sessions(
        &self,
        user_id: &str,
    ) -> Result<(), SessionError> {
        let mut conn = self.client.get_async_connection().await?;

        // Scan for user's sessions
        let pattern = format!("{}:*", self.prefix);
        let keys: Vec<String> = conn.keys(pattern).await?;

        for key in keys {
            if let Some(data) = self.get_session(&key).await? {
                if data.user_id == user_id {
                    conn.del(&key).await?;
                }
            }
        }

        Ok(())
    }
}
```

---

### 6.2 Session Security

#### Secure Session Configuration

```rust
use cookie::{Cookie, SameSite};

fn create_session_cookie(
    session_id: &str,
    max_age: Option<i64>,
    secure: bool,
) -> Cookie<'static> {
    Cookie::build("llm_cost_ops_session", session_id.to_owned())
        .path("/")
        .secure(secure)              // HTTPS only in production
        .http_only(true)             // Prevent JavaScript access
        .same_site(SameSite::Lax)    // CSRF protection
        .max_age(max_age.map(|s| cookie::time::Duration::seconds(s)))
        .finish()
}

// Session fixation prevention
async fn regenerate_session_id(
    old_session_id: &str,
    session_store: &RedisSessionStore,
) -> Result<String, SessionError> {
    // Get existing session data
    let data = session_store.get_session(old_session_id).await?
        .ok_or(SessionError::SessionNotFound)?;

    // Generate new session ID
    let new_session_id = generate_secure_session_id();

    // Create new session with same data
    session_store.create_session(&new_session_id, data, None).await?;

    // Delete old session
    session_store.revoke_session(old_session_id).await?;

    Ok(new_session_id)
}

fn generate_secure_session_id() -> String {
    use rand::Rng;
    use sha2::{Sha256, Digest};

    let random_bytes: Vec<u8> = (0..32)
        .map(|_| rand::thread_rng().gen::<u8>())
        .collect();

    let mut hasher = Sha256::new();
    hasher.update(&random_bytes);
    hasher.update(Utc::now().timestamp().to_string().as_bytes());

    let hash = hasher.finalize();
    base64::encode_config(&hash, base64::URL_SAFE_NO_PAD)
}
```

---

## 7. Compliance Requirements

### 7.1 SOC 2 Type II Requirements

#### Access Control Requirements

**CC6.1 - Logical and Physical Access Controls**

```rust
// Implement least privilege access
#[derive(Debug)]
struct AccessControlPolicy {
    // Require MFA for privileged accounts
    mfa_required_roles: Vec<String>,

    // Session timeout policies
    max_session_duration: Duration,
    idle_timeout: Duration,

    // Password policies (if using password auth)
    password_min_length: usize,
    password_require_complexity: bool,

    // Account lockout
    max_failed_attempts: u32,
    lockout_duration: Duration,
}

impl AccessControlPolicy {
    fn enforce_soc2_controls() -> Self {
        Self {
            mfa_required_roles: vec![
                "super_admin".to_string(),
                "org_admin".to_string(),
                "auditor".to_string(),
            ],
            max_session_duration: Duration::hours(8),
            idle_timeout: Duration::minutes(30),
            password_min_length: 12,
            password_require_complexity: true,
            max_failed_attempts: 5,
            lockout_duration: Duration::minutes(30),
        }
    }
}
```

**CC6.2 - Prior to Issuing Credentials**

```rust
async fn verify_user_before_provisioning(
    user_email: &str,
    sso_attributes: &SsoAttributes,
) -> Result<(), ComplianceError> {
    // Verify user is from approved domain
    verify_approved_domain(user_email)?;

    // Check against external directory (if available)
    verify_employment_status(user_email).await?;

    // Require email verification
    if !sso_attributes.email_verified {
        return Err(ComplianceError::EmailNotVerified);
    }

    Ok(())
}
```

**CC6.3 - Removes Access**

```rust
async fn automated_access_removal(
    user_id: &str,
    termination_event: TerminationEvent,
) -> Result<(), ComplianceError> {
    // Immediate actions
    session_manager.revoke_all_sessions(user_id).await?;
    api_key_manager.revoke_all_keys(user_id).await?;
    user_repo.deactivate_user(user_id).await?;

    // Audit trail
    audit_log(AuditEvent {
        event_type: AuditEventType::AccessRevoked,
        user_id: Some(user_id.to_string()),
        details: format!("Automated revocation: {:?}", termination_event),
        severity: AuditSeverity::Critical,
        timestamp: Utc::now(),
    }).await?;

    // Notify security team
    notify_security_team(&termination_event).await?;

    Ok(())
}
```

---

### 7.2 GDPR Compliance

#### Data Subject Rights

**Right to Access (Article 15)**

```rust
async fn export_user_data(
    user_id: &str,
    user_repo: &dyn UserRepository,
) -> Result<GdprDataExport, GdprError> {
    let user = user_repo.find_by_id(user_id).await?
        .ok_or(GdprError::UserNotFound)?;

    // Collect all personal data
    let export = GdprDataExport {
        user_profile: user.clone(),
        authentication_logs: get_auth_logs(user_id).await?,
        sessions: get_active_sessions(user_id).await?,
        api_keys: get_api_keys(user_id).await?,
        audit_trail: get_user_audit_trail(user_id).await?,

        // Include related data
        cost_records: get_user_cost_records(user_id).await?,
        usage_records: get_user_usage_records(user_id).await?,

        export_date: Utc::now(),
    };

    Ok(export)
}
```

**Right to Erasure (Article 17)**

```rust
async fn delete_user_data(
    user_id: &str,
    erasure_request: ErasureRequest,
) -> Result<ErasureCertificate, GdprError> {
    // Verify legal basis for deletion
    verify_erasure_legality(&erasure_request)?;

    // Pseudonymize data that must be retained
    pseudonymize_cost_records(user_id).await?;
    pseudonymize_audit_logs(user_id).await?;

    // Delete personal data
    user_repo.hard_delete(user_id).await?;
    session_manager.revoke_all_sessions(user_id).await?;
    api_key_manager.delete_all_keys(user_id).await?;

    // Generate certificate of deletion
    let certificate = ErasureCertificate {
        user_id: user_id.to_string(),
        deletion_timestamp: Utc::now(),
        data_categories_deleted: vec![
            "user_profile",
            "authentication_credentials",
            "sessions",
            "api_keys",
        ],
        data_categories_pseudonymized: vec![
            "cost_records",
            "audit_logs",
        ],
        retention_reason: "Legal obligation (financial records)".to_string(),
    };

    Ok(certificate)
}
```

**Data Minimization (Article 5.1.c)**

```rust
#[derive(Debug, Serialize, Deserialize)]
struct MinimalUserProfile {
    user_id: String,
    email: String,
    organization_id: String,
    roles: Vec<String>,
    // Only essential fields
}

// Don't store unnecessary PII
impl From<SsoAttributes> for MinimalUserProfile {
    fn from(attrs: SsoAttributes) -> Self {
        Self {
            user_id: uuid::Uuid::new_v4().to_string(),
            email: attrs.email,
            organization_id: attrs.organization_id,
            roles: attrs.groups,
            // DO NOT store: phone number, address, date of birth, etc.
        }
    }
}
```

---

### 7.3 Audit Logging for Compliance

#### Comprehensive Audit Events

```rust
#[derive(Debug, Serialize, Deserialize)]
enum AuditEventType {
    // Authentication events
    LoginAttempt,
    LoginSuccess,
    LoginFailure,
    Logout,
    SessionExpired,

    // User lifecycle
    UserProvisioned,
    UserDeprovisioned,
    UserUpdated,
    RoleAssigned,
    RoleRevoked,

    // Access control
    PermissionGranted,
    PermissionDenied,
    AccessRevoked,

    // Configuration changes
    SsoConfigUpdated,
    IdpAdded,
    IdpRemoved,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuditEvent {
    event_id: String,
    event_type: AuditEventType,
    timestamp: DateTime<Utc>,
    user_id: Option<String>,
    organization_id: Option<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    resource: Option<String>,
    action: Option<String>,
    result: AuditResult,
    severity: AuditSeverity,
    details: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
enum AuditResult {
    Success,
    Failure { reason: String },
    Denied { reason: String },
}

#[derive(Debug, Serialize, Deserialize)]
enum AuditSeverity {
    Info,
    Warning,
    Critical,
}

// Immutable audit log storage
async fn write_audit_log(
    event: AuditEvent,
    audit_store: &dyn AuditStore,
) -> Result<(), AuditError> {
    // Write to append-only log
    audit_store.append(event.clone()).await?;

    // Send to SIEM (Security Information and Event Management)
    send_to_siem(&event).await?;

    // Alert on critical events
    if event.severity == AuditSeverity::Critical {
        alert_security_team(&event).await?;
    }

    Ok(())
}
```

---

## 8. Implementation Roadmap

### Phase 1: Foundation (Weeks 1-4)

**Week 1-2: Core SSO Infrastructure**
- [ ] SAML 2.0 library integration (samael or opensaml)
- [ ] OAuth 2.0/OIDC library integration (oauth2-rs)
- [ ] State management for auth flows
- [ ] Redirect URI validation
- [ ] CSRF protection

**Week 3-4: First IdP Integration**
- [ ] Okta SAML 2.0 integration
- [ ] Okta OIDC integration
- [ ] JIT user provisioning
- [ ] Basic attribute mapping
- [ ] Integration tests

**Deliverables:**
- Working SSO login with Okta
- JIT user creation
- Session management
- Documentation

---

### Phase 2: Multi-IdP Support (Weeks 5-8)

**Week 5-6: Additional IdP Integrations**
- [ ] Azure AD / Entra ID (SAML + OIDC)
- [ ] Google Workspace (OIDC)
- [ ] Auth0 (OIDC)
- [ ] GitHub Enterprise (OAuth 2.0)

**Week 7-8: Advanced Provisioning**
- [ ] SCIM 2.0 endpoint implementation
- [ ] User lifecycle management
- [ ] Group synchronization
- [ ] Attribute mapping configuration UI

**Deliverables:**
- 5+ IdP integrations
- SCIM provisioning
- Admin UI for IdP configuration

---

### Phase 3: Enterprise Features (Weeks 9-12)

**Week 9-10: Security Enhancements**
- [ ] MFA enforcement
- [ ] Advanced session management
- [ ] Adaptive session timeouts
- [ ] IP whitelisting

**Week 11-12: Compliance & Audit**
- [ ] Comprehensive audit logging
- [ ] SOC 2 controls implementation
- [ ] GDPR data export/deletion
- [ ] Compliance reporting

**Deliverables:**
- Production-ready SSO system
- SOC 2 compliance documentation
- GDPR compliance features

---

### Phase 4: Testing & Hardening (Weeks 13-16)

**Week 13-14: Security Testing**
- [ ] Penetration testing
- [ ] Security code review
- [ ] Vulnerability scanning
- [ ] OWASP top 10 validation

**Week 15-16: Performance & Reliability**
- [ ] Load testing
- [ ] Failover testing
- [ ] Disaster recovery testing
- [ ] Performance optimization

**Deliverables:**
- Security audit report
- Performance benchmarks
- Runbook for operations

---

## 9. References & Resources

### 9.1 Standards & Specifications

**SAML 2.0**
- OASIS SAML 2.0 Core: https://docs.oasis-open.org/security/saml/v2.0/saml-core-2.0-os.pdf
- SAML 2.0 Profiles: https://docs.oasis-open.org/security/saml/v2.0/saml-profiles-2.0-os.pdf
- SAML 2.0 Bindings: https://docs.oasis-open.org/security/saml/v2.0/saml-bindings-2.0-os.pdf

**OAuth 2.0 / OpenID Connect**
- RFC 6749 - OAuth 2.0: https://datatracker.ietf.org/doc/html/rfc6749
- RFC 7636 - PKCE: https://datatracker.ietf.org/doc/html/rfc7636
- OpenID Connect Core: https://openid.net/specs/openid-connect-core-1_0.html
- OpenID Connect Discovery: https://openid.net/specs/openid-connect-discovery-1_0.html

**SCIM 2.0**
- RFC 7643 - SCIM Core Schema: https://datatracker.ietf.org/doc/html/rfc7643
- RFC 7644 - SCIM Protocol: https://datatracker.ietf.org/doc/html/rfc7644

**Security Best Practices**
- OWASP Authentication Cheat Sheet: https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html
- NIST Digital Identity Guidelines: https://pages.nist.gov/800-63-3/

---

### 9.2 Rust Libraries

**SAML**
- `samael` - SAML 2.0 library for Rust
- `opensaml` - Alternative SAML implementation

**OAuth / OIDC**
- `oauth2` - OAuth 2.0 client library
- `openidconnect` - OpenID Connect library
- `jsonwebtoken` - JWT encoding/decoding

**HTTP & Web**
- `axum` - Web framework (current)
- `tower` - Middleware
- `reqwest` - HTTP client

**Session Management**
- `redis` - Redis client
- `tower-sessions` - Session middleware

---

### 9.3 IdP Documentation

**Okta**
- SAML Setup: https://developer.okta.com/docs/guides/build-sso-integration/saml2/main/
- OIDC Setup: https://developer.okta.com/docs/guides/implement-grant-type/authcode/main/
- SCIM Provisioning: https://developer.okta.com/docs/guides/scim-provisioning-integration-overview/main/

**Azure AD**
- Enterprise Apps: https://learn.microsoft.com/en-us/entra/identity/enterprise-apps/
- SAML Protocol: https://learn.microsoft.com/en-us/entra/identity-platform/saml-protocol-reference
- Microsoft Identity Platform: https://learn.microsoft.com/en-us/entra/identity-platform/

**Google Workspace**
- OAuth 2.0: https://developers.google.com/identity/protocols/oauth2
- OpenID Connect: https://developers.google.com/identity/openid-connect/openid-connect

**Auth0**
- Universal Login: https://auth0.com/docs/authenticate/login/auth0-universal-login
- OIDC Protocol: https://auth0.com/docs/authenticate/protocols/openid-connect-protocol

**GitHub**
- OAuth Apps: https://docs.github.com/en/apps/oauth-apps/building-oauth-apps

---

### 9.4 Compliance Resources

**SOC 2**
- AICPA TSC: https://www.aicpa.org/resources/article/trust-services-criteria
- SOC 2 Compliance Guide: https://www.imperva.com/learn/data-security/soc-2-compliance/

**GDPR**
- Official Text: https://gdpr-info.eu/
- ICO Guidance: https://ico.org.uk/for-organisations/guide-to-data-protection/

---

## Appendix A: Attribute Mapping Templates

### Okta → LLM-CostOps

```yaml
saml_attributes:
  email:
    source: "user.email"
    target: "email"
    required: true

  first_name:
    source: "user.firstName"
    target: "first_name"
    required: false

  last_name:
    source: "user.lastName"
    target: "last_name"
    required: false

  organization:
    source: "user.organization"
    target: "organization_id"
    transform: "lowercase_and_slugify"
    required: true

  groups:
    source: "appuser.groups"
    target: "roles"
    array: true
    mapping:
      "LLM-CostOps-Admins": "org_admin"
      "LLM-CostOps-Billing": "billing"
      "LLM-CostOps-ReadOnly": "read_only"
```

### Azure AD → LLM-CostOps

```yaml
saml_attributes:
  email:
    source: "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress"
    target: "email"
    required: true

  name:
    source: "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/name"
    target: "full_name"

  groups:
    source: "http://schemas.microsoft.com/ws/2008/06/identity/claims/groups"
    target: "roles"
    array: true
```

---

## Appendix B: Sample Configurations

### SSO Configuration File

```yaml
# config/sso.yaml

sso:
  enabled: true
  default_provider: "okta"

  providers:
    okta:
      type: "saml"
      entity_id: "llm-cost-ops"
      acs_url: "https://llm-cost-ops.example.com/saml/acs"
      metadata_url: "https://your-tenant.okta.com/app/abc123/sso/saml/metadata"
      sign_requests: true
      want_assertions_signed: true

    azure_ad:
      type: "oidc"
      issuer: "https://login.microsoftonline.com/{tenant}/v2.0"
      client_id: "your-client-id"
      client_secret: "${AZURE_CLIENT_SECRET}"
      scopes: ["openid", "profile", "email", "offline_access"]

    google:
      type: "oidc"
      issuer: "https://accounts.google.com"
      client_id: "your-client-id.apps.googleusercontent.com"
      client_secret: "${GOOGLE_CLIENT_SECRET}"
      hosted_domain: "example.com"  # Restrict to Workspace domain

  provisioning:
    jit_enabled: true
    scim_enabled: true
    default_role: "read_only"
    auto_activate: true
    require_email_verification: false

  session:
    default_lifetime_secs: 28800  # 8 hours
    idle_timeout_secs: 1800       # 30 minutes
    max_lifetime_secs: 86400      # 24 hours

  security:
    mfa_required: true
    mfa_required_roles: ["super_admin", "org_admin"]
    allowed_domains: ["example.com"]
```

---

## Document Approval

**Research Completed:** 2025-11-15
**Author:** SSO Standards & Research Specialist
**Status:** Final

**Next Steps:**
1. Review by Architecture team
2. Security review
3. Implementation planning
4. Resource allocation

---

**End of Enterprise SSO Research Report**
