# aws is in [providers], but here it is declared without a version: nothing to enforce, so it
# must not be flagged (the rule is "if a version is set, it must match").
terraform {
  required_providers {
    aws = {
      source = "hashicorp/aws"
    }
  }
}

provider "aws" {
  region = "us-west-1"
}
