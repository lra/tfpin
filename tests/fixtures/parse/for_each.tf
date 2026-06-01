# Modules with for_each and interpolated arguments. The source is not pinned in [modules], so the
# module check ignores it, but the file must still parse cleanly.
module "environment_workload_secrets" {
  for_each = { for secret in local.flatten_secrets : "${secret.workload}.${secret.secret}" => secret }

  providers = {
    aws = aws.acme-dev
  }

  source = "../modules/aws-sm-workload-secret"
  path   = "/workload/dev/${each.value.workload}/${each.value.secret}"
}
