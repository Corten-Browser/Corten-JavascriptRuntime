# Contract Templates

This directory contains API contract templates for inter-component communication.

## Purpose

Components in the orchestration system communicate exclusively through defined contracts. These templates provide starting points for creating OpenAPI/YAML specifications that define:

- API endpoints
- Request/response schemas
- Data types
- Error responses
- Authentication requirements

## Template Files

### rest-api-template.yaml
Comprehensive RESTful API contract template (OpenAPI 3.0) with:
- **CRUD Operations**: Complete Create, Read, Update, Delete endpoints
- **HTTP Methods**: GET, POST, PUT, PATCH, DELETE
- **Pagination**: List endpoints with page/size parameters
- **Filtering & Sorting**: Query parameters for data retrieval
- **Health Checks**: `/health` and `/health/ready` endpoints
- **Authentication**: Bearer token (JWT) security scheme
- **Error Handling**: Comprehensive error response schemas
- **Response Codes**: 200, 201, 204, 400, 401, 404, 409, 500
- **Variable Substitution**: {{COMPONENT_NAME}}, {{RESOURCE_NAME}}, etc.

**Use for**: REST APIs, HTTP services, web APIs

### grpc-service-template.proto
gRPC service contract template (Protocol Buffers 3) with:
- **Service Definition**: Complete gRPC service structure
- **RPC Methods**: Unary, server streaming, client streaming, bidirectional
- **CRUD Operations**: Create, Get, List, Update, Delete RPCs
- **Batch Operations**: BatchCreate, BatchGet for efficiency
- **Streaming**: Watch methods for real-time updates
- **Health Check**: Standard gRPC health checking
- **Pagination**: Page token-based pagination
- **Field Masks**: Partial field selection
- **Error Handling**: Structured error messages
- **Variable Substitution**: {{SERVICE_NAME}}, {{RESOURCE_CLASS}}, etc.

**Use for**: High-performance services, microservices, real-time communication

### event-bus-template.yaml
Event-driven architecture contract template (AsyncAPI 2.6) with:
- **Event Channels**: Created, Updated, Deleted, StatusChanged
- **Command Channels**: Create, Update, Delete commands
- **Domain Events**: Past-tense event naming (Created, Updated)
- **Commands**: Imperative command naming (Create, Update)
- **Event Headers**: Correlation IDs, event types, timestamps
- **Error Events**: Dedicated error event channel
- **Streaming**: Subscribe/publish patterns
- **Message Protocols**: Kafka, AMQP, MQTT support
- **Security**: SASL/SCRAM authentication
- **Correlation**: Full distributed tracing support
- **Variable Substitution**: {{COMPONENT_NAME}}, {{RESOURCE_NAME}}, etc.

**Use for**: Event sourcing, CQRS, pub/sub messaging, async processing

### data-contract-template.yaml
Comprehensive data contract template with:
- **Database Schema**: Tables, columns, indexes, constraints
- **Access Patterns**: Query patterns with performance metrics
- **Data Quality**: Validation, completeness, accuracy, consistency rules
- **Transformations**: ETL/ELT pipeline definitions
- **Data Lineage**: Upstream sources and downstream consumers
- **SLA Definitions**: Availability, performance, data quality targets
- **Security**: Encryption, access control, PII handling
- **Monitoring**: Metrics, alerts, quality checks
- **Retention**: Data retention and archival policies
- **Testing**: Unit tests, integration tests, fixtures
- **Variable Substitution**: {{TABLE_NAME}}, {{FIELD_NAME}}, etc.

**Use for**: Data layers, databases, data warehouses, data quality

## Contract Naming Convention

Contracts should be named: `{component-name}-api.yaml`

Examples:
- `user-service-api.yaml`
- `payment-api.yaml`
- `notification-service-api.yaml`

## OpenAPI Template Structure

```yaml
openapi: 3.0.0
info:
  title: Component API
  version: 1.0.0
  description: API contract for [component]

servers:
  - url: http://localhost:8000
    description: Development server

paths:
  /resource:
    get:
      summary: Get resource
      operationId: getResource
      responses:
        '200':
          description: Success
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Resource'
        '404':
          description: Not found

components:
  schemas:
    Resource:
      type: object
      required:
        - id
      properties:
        id:
          type: string
          format: uuid
        name:
          type: string

  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
```

## Usage in Components

Components reference their contracts in their CLAUDE.md:

```markdown
## Integration Points
- Read your API contract from: ../../contracts/user-service-api.yaml
- Implement all endpoints defined in the contract
- Use shared libraries for common functionality
```

## Contract Validation

### Built-in Validator

The orchestration system includes a contract validator that supports all template types:

```bash
# Validate a single contract
python orchestration/contract_validator.py validate contracts/my-api.yaml

# Validate all contracts in directory
python orchestration/contract_validator.py validate-all

# Validate with strict mode (additional checks)
python orchestration/contract_validator.py validate --strict contracts/my-api.yaml

# Check for unreplaced template variables
python orchestration/contract_validator.py check-variables contracts/my-api.yaml
```

**Supported Contract Types:**
- OpenAPI 3.0 (REST APIs)
- Protocol Buffers 3 (gRPC)
- AsyncAPI 2.6 (Event Bus)
- Data Contracts (YAML)

**Validation Checks:**
- Required fields present
- Valid YAML/Proto syntax
- Proper versioning
- Schema completeness
- Unreplaced template variables

### External Validators (Optional)

For additional validation, you can use:

```bash
# OpenAPI validator
npx @stoplight/spectral-cli lint contracts/my-api.yaml

# Swagger CLI
swagger-cli validate contracts/my-api.yaml

# AsyncAPI validator
asyncapi validate contracts/my-events.yaml

# Protocol Buffers compiler
protoc --proto_path=contracts --lint_out=. contracts/my-service.proto
```

## Versioning

When making breaking changes to contracts:

1. Create a new version: `user-service-api-v2.yaml`
2. Update component references
3. Maintain backward compatibility during transition
4. Archive old versions when no longer needed

## Best Practices

1. **Define Before Implementation**: Create contracts before writing code
2. **Keep Contracts Minimal**: Only include necessary endpoints
3. **Document Thoroughly**: Use descriptions for all fields
4. **Version Explicitly**: Include version in contract and filename
5. **Validate Regularly**: Run validation in CI/CD
6. **Review Changes**: Contracts changes require orchestrator approval
7. **Use Examples**: Include example requests/responses
8. **Error Handling**: Define all error cases

## Contract-Driven Development Workflow

1. **Design Phase**: Create contract defining the API
2. **Review Phase**: Orchestrator reviews and approves contract
3. **Implementation Phase**: Sub-agent implements per contract
4. **Testing Phase**: Validate implementation matches contract
5. **Integration Phase**: Connect components via contracts

## Tools

Recommended tools for contract development:

- **Swagger Editor**: Edit and visualize OpenAPI contracts
- **Stoplight Studio**: Design and document APIs
- **Postman**: Test API contracts
- **OpenAPI Generator**: Generate client/server code from contracts
