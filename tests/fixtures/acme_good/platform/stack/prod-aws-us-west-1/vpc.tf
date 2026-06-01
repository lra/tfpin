# A registry module whose source IS pinned in [modules], with the correct version.
module "vpc" {
  source  = "terraform-aws-modules/vpc/aws"
  version = "~> 6.2"

  name = "prod"
  cidr = "10.0.0.0/16"
}
