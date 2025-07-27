# Frontend Integration Guide

This guide shows how to integrate FreshAPI with your TypeScript/Vue.js frontend using automatic schema synchronization.

## 🔄 Schema Synchronization

### Available Endpoints (Development Only)

- **GraphQL SDL**: `GET /schema.graphql` - Schema Definition Language format
- **JSON Introspection**: `GET /schema.json` - Full introspection data for codegen
- **GraphQL Playground**: `GET /playground` - Interactive API explorer

⚠️ **Security Note**: These endpoints are only available when `ENVIRONMENT=development`

## 🛠 Frontend Setup

### 1. Install Dependencies

```bash
npm install @graphql-codegen/cli @graphql-codegen/typescript @graphql-codegen/typescript-operations @graphql-codegen/typescript-vue-apollo
```

### 2. Create GraphQL Codegen Config

Create `codegen.yml`:

```yaml
schema: "http://localhost:8080/graphql"
documents: "src/**/*.{vue,ts,js}"
generates:
  src/generated/graphql.ts:
    plugins:
      - typescript
      - typescript-operations
    config:
      useIndexSignature: true
      skipTypename: false
      withHooks: true
```

### 3. Add Package.json Scripts

```json
{
  "scripts": {
    "codegen": "graphql-codegen --config codegen.yml",
    "codegen:watch": "graphql-codegen --watch",
    "schema:pull": "curl -o schema.graphql http://localhost:8080/schema.graphql"
  }
}
```

### 4. Development Workflow

```bash
# Start your API in development
cd freshapi
cargo run

# In your frontend project
npm run codegen:watch  # Auto-regenerate types on schema changes
```

## 🚀 Production Deployment

### Railway Environment Variables

Set these in your Railway project:

```bash
# Required
DATABASE_URL=postgresql://... # Railway provides this automatically
JWT_SECRET=your-production-jwt-secret-here

# Optional
ENVIRONMENT=production
HOST=0.0.0.0
PORT=8080
RESEND_API_KEY=your-resend-key

# Admin seeding (only for initial setup)
ADMIN_EMAIL=admin@yourdomain.com
ADMIN_PASSWORD=secure-admin-password
```

### Frontend Production Build

For production, use a static schema file:

```bash
# Generate schema during CI/CD
curl -o src/schema.graphql https://your-api-dev.railway.app/schema.graphql
npm run codegen
npm run build
```

## 🔐 Security Features

- ✅ **Introspection disabled** in production
- ✅ **Schema endpoints blocked** in production  
- ✅ **Environment-based access control**
- ✅ **CORS configured** for your frontend domains

## 📚 Example Usage

### Apollo Client Setup

```typescript
import { ApolloClient, InMemoryCache, createHttpLink } from '@apollo/client'
import { setContext } from '@apollo/client/link/context'

const httpLink = createHttpLink({
  uri: process.env.VUE_APP_GRAPHQL_URL || 'http://localhost:8080/graphql',
})

const authLink = setContext((_, { headers }) => {
  const token = localStorage.getItem('token')
  return {
    headers: {
      ...headers,
      authorization: token ? `Bearer ${token}` : "",
    }
  }
})

export const apolloClient = new ApolloClient({
  link: authLink.concat(httpLink),
  cache: new InMemoryCache(),
})
```

### Generated Types Usage

```typescript
import { LoginMutation, RegisterMutation } from '@/generated/graphql'

// Fully typed mutations and queries
const loginUser = async (email: string, password: string) => {
  const result = await apolloClient.mutate<LoginMutation>({
    mutation: LOGIN_MUTATION,
    variables: { input: { email, password } }
  })
  return result.data?.login
}
```

## 🔄 Schema Evolution

When you add new GraphQL types/mutations:

1. ✅ **No manual updates needed** - introspection handles everything
2. ✅ **Run `npm run codegen`** to get new TypeScript types  
3. ✅ **Use new types immediately** in your frontend code

The schema synchronization is completely automatic! 🎉