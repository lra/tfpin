# configuration_aliases is an array of identifier expressions with a trailing comma, alongside a
# normal version pin (which must still be read and validated correctly).
terraform {
  required_version = "~> 1.15.5"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.24"
      configuration_aliases = [
        aws.acme-dev, # AWS acme-dev us-east-1
      ]
    }
  }
}
