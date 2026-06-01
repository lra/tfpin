# This file lives under .terraform/ and must NEVER be scanned. It deliberately contains a wrong
# required_version: if the default exclusion failed, the "good" tree would report a violation.
terraform {
  required_version = "= 0.11.0"
}
