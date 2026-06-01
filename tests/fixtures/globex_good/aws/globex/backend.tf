terraform {
  backend "s3" {
    bucket              = "globex-terraform-state"
    key                 = "aws/globex/terraform.tfstate"
    region              = "us-east-1"
    encrypt             = true
    kms_key_id          = "alias/globex-terraform-state"
    use_lockfile        = true
    allowed_account_ids = ["222222222222"]
    profile             = "globex"
  }
}
