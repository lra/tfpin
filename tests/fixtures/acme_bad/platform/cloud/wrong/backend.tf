terraform {
  backend "s3" {
    # Every attribute below is wrong, plus the key does not match the workspace path.
    allowed_account_ids = ["000000000000"]
    bucket              = "some-other-bucket"
    key                 = "wrong/key"
    region              = "us-east-1"
    use_lockfile        = false
  }
}
