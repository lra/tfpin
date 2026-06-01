# Heredocs (both <<- and <<) in attribute values must not break parsing.
resource "strongdm_policy" "example" {
  name   = "example"
  policy = <<-EOP
    permit where resource.tags.env == "dev"
    permit where resource.tags.team == "platform"
  EOP
}

resource "helm_release" "ci" {
  name = "ci"
  values = [<<EOF
replicaCount: 2
image:
  tag: latest
EOF
  ]
}
