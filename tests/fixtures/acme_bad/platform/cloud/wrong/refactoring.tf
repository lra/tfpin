# Leftover refactoring blocks: all three should be flagged.
moved {
  from = aws_s3_bucket.old
  to   = aws_s3_bucket.new
}

import {
  to = aws_s3_bucket.imported
  id = "my-bucket"
}

removed {
  from = aws_s3_bucket.gone

  lifecycle {
    destroy = false
  }
}
