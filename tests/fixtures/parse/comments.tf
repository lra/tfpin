# Comments interleaved in blocks, and attribute values that are variable traversals (var.*),
# function calls and conditionals — none of which are literal strings.
provider "oci" {
  # "Acme (GCP)" Elastic account: 1234567890
  apikey = var.EC_API_KEY_GCP # trailing comment
}

provider "azurerm" {
  features {}
  tenant_id       = "63049d9f-ad4a-40e1-9041-d096a0f29d1e" # ad-azure-dev
  subscription_id = coalesce(var.subscription_id, "default")
}
