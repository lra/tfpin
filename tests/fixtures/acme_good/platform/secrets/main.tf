# A local module whose source is not in [modules]: ignored. Even if its source were pinned, it
# declares no version, so it would be skipped rather than flagged.
module "acme_regions" {
  source = "../modules/acme-regions"
}
