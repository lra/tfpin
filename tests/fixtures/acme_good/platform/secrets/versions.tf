# The terraform block is split across files in this directory: required_version lives here,
# the backend in backend.tf, and required_providers in providers.tf. Each file is checked
# independently and must not be flagged for the parts it does not declare.
terraform {
  required_version = "~> 1.15.5"
}
