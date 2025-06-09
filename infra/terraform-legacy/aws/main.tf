# https://aws.amazon.com/blogs/big-data/introducing-terraform-support-for-amazon-opensearch-ingestion/
# https://docs.aws.amazon.com/opensearch-service/latest/developerguide/osis-get-started.html
# https://docs.aws.amazon.com/opensearch-service/latest/developerguide/creating-pipeline.html#pipeline-blueprint
terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.36"
    }
  }

  required_version = ">= 1.2.0"
}

provider "aws" {
  region = "us-east-1"
}

data "aws_region" "current" {}
data "aws_caller_identity" "current" {}
locals {
    account_id = data.aws_caller_identity.current.account_id
}

output "ingest_endpoint_url" {
  value = tolist(aws_osis_pipeline.example.ingest_endpoint_urls)[0]
}

resource "aws_iam_role" "example" {
  name = "exampleosisrole"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Sid    = ""
        Principal = {
          Service = "osis-pipelines.amazonaws.com"
        }
      },
    ]
  })
}

resource "aws_opensearch_domain" "test" {
  domain_name           = "osi-example-domain"
  engine_version        = "OpenSearch_2.7"
  cluster_config {
    instance_type = "t3.small.search"
    instance_count = 1
  }
  encrypt_at_rest {
    enabled = true
  }
  domain_endpoint_options {
    enforce_https       = true
    tls_security_policy = "Policy-Min-TLS-1-2-2019-07"
  }
  node_to_node_encryption {
    enabled = true
  }
  ebs_options {
    ebs_enabled = true
    volume_size = 10
    volume_type = "gp2"
  }
  access_policies = <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": {
        "AWS": "${aws_iam_role.example.arn}"
      },
      "Action": "es:*"
    }
  ]
}
EOF
}

resource "aws_iam_policy" "example" {
  name = "osis_role_policy"
  description = "Policy for OSIS pipeline role"
  policy = jsonencode({
    Version = "2012-10-17",
    Statement = [
        {
          Action = ["es:DescribeDomain"]
          Effect = "Allow"
          Resource = "arn:aws:es:${data.aws_region.current.name}:${local.account_id}:domain/*"
        },
        {
          Action = ["es:ESHttp*"]
          Effect = "Allow"
          Resource = "arn:aws:es:${data.aws_region.current.name}:${local.account_id}:domain/osi-test-domain/*"
        }
    ]
})
}

resource "aws_iam_role_policy_attachment" "example" {
  role       = aws_iam_role.example.name
  policy_arn = aws_iam_policy.example.arn
}

resource "aws_cloudwatch_log_group" "example" {
  name = "/aws/vendedlogs/OpenSearchIngestion/example-pipeline"
  retention_in_days = 365
  tags = {
    Name = "AWS Blog OSIS Pipeline Example"
  }
}

resource "aws_osis_pipeline" "example" {
  pipeline_name               = "example-pipeline"
  pipeline_configuration_body = <<-EOT
            version: "2"
            example-pipeline:
              source:
                http:
                  path: "/test_ingestion_path"
              processor:
                - date:
                    from_time_received: true
                    destination: "@timestamp"
              sink:
                - opensearch:
                    hosts: ["https://${aws_opensearch_domain.test.endpoint}"]
                    index: "solana_validator_logs"
                    aws:
                      sts_role_arn: "${aws_iam_role.example.arn}"   
                      region: "${data.aws_region.current.name}"
        EOT
  max_units                   = 1
  min_units                   = 1
  log_publishing_options {
    is_logging_enabled = true
    cloudwatch_log_destination {
      log_group = aws_cloudwatch_log_group.example.name
    }
  }
}