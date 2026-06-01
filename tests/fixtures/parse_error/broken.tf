# Deliberately malformed HCL: unbalanced braces and a dangling attribute. tfpin must report a
# per-file parse error and exit non-zero, not panic.
terraform {
  required_version = "~> 1.15.5"

resource "aws_s3_bucket" {
  bucket =
