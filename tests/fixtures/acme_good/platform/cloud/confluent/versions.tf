terraform {
  required_version = "~> 1.15.5"

  required_providers {
    confluent = {
      source  = "confluentinc/confluent"
      version = "~> 2.9"
    }
  }
}
