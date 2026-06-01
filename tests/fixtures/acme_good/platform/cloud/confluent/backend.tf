terraform {
  backend "s3" {
    allowed_account_ids = ["111111111111"]
    bucket              = "acme-terraform-state"
    key                 = "acme/infra/platform/cloud/confluent"
    region              = "us-west-1"
    use_lockfile        = true
  }
}
