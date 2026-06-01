module "vpc" {
  source = "terraform-aws-modules/vpc/aws"
  # Wrong: expected "~> 6.2".
  version = "~> 5.0"
}
