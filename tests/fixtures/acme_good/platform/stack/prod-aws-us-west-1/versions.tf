terraform {
  required_version = "~> 1.15.5"

  required_providers {
    # Compound version constraint, compared verbatim as a single string.
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.37, >= 2.37.1"
    }
    # google is in [providers] but declared without a version here: skipped.
    google = {
      source = "hashicorp/google"
    }
  }
}
