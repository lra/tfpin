terraform {
  required_version = "~> 1.15.5"

  # Multiple providers in a single multi-line required_providers block.
  required_providers {
    github = {
      source  = "integrations/github"
      version = "~> 6.12"
    }
    hcloud = {
      source  = "hetznercloud/hcloud"
      version = "~> 1.63"
    }
    tailscale = {
      source  = "tailscale/tailscale"
      version = "~> 0.29"
    }
  }
}
