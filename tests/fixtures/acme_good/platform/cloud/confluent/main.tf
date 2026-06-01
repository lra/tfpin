# A plain resource file with none of the checked blocks: nothing to validate here.
resource "confluent_environment" "main" {
  display_name = "production"
}

# A provider that is NOT listed in [providers] must be ignored, not flagged.
terraform {
  required_providers {
    random = {
      source  = "hashicorp/random"
      version = "~> 3.6"
    }
  }
}
