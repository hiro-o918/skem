# skem Examples

This directory contains example configurations and schema files that demonstrate how to use skem.

## Example Configuration

`.skem.yaml` - Configuration file that demonstrates downloading multiple types of schema files:
- Proto files from `schemas/proto/`
- OpenAPI specifications from `schemas/openapi/`

### Usage

```bash
cd examples
skem sync   # Synchronize all dependencies
ls vendor/proto/    # Check downloaded proto files
ls vendor/openapi/  # Check downloaded OpenAPI specs
```

## Schema Examples

### schemas/proto/

Sample Protocol Buffer definitions demonstrating:
- Message definitions
- Service definitions
- RPC operations

Files:
- `user.proto` - User message and UserService definition

### schemas/openapi/

Sample OpenAPI 3.0 specifications demonstrating:
- API endpoints
- Request/response schemas
- Component reusability

Files:
- `api.yaml` - REST API specification with user endpoints
