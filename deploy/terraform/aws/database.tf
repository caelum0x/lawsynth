# RDS Postgres instance backing LawSynth metadata: run specs, artifact
# references, and append-only run events.

resource "random_password" "db" {
  length  = 32
  special = false
}

# Store the generated master password in Secrets Manager for retrieval by the
# application (referenced from the Helm chart's externalServices.postgres).
resource "aws_secretsmanager_secret" "db" {
  name        = "${local.name}/postgres"
  description = "LawSynth metadata database master credentials."
  tags        = local.tags
}

resource "aws_secretsmanager_secret_version" "db" {
  secret_id = aws_secretsmanager_secret.db.id
  secret_string = jsonencode({
    username = var.db_username
    password = random_password.db.result
    dbname   = var.db_name
    host     = module.db.db_instance_address
    port     = module.db.db_instance_port
  })
}

resource "aws_security_group" "db" {
  name        = "${local.name}-db"
  description = "Allow Postgres access from the EKS nodes only."
  vpc_id      = module.vpc.vpc_id

  ingress {
    description     = "Postgres from EKS nodes"
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [module.eks.node_security_group_id]
  }

  egress {
    description = "Allow all outbound"
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = local.tags
}

module "db" {
  source  = "terraform-aws-modules/rds/aws"
  version = "~> 6.7"

  identifier = "${local.name}-pg"

  engine               = "postgres"
  engine_version       = var.postgres_version
  family               = "postgres16"
  major_engine_version = "16"
  instance_class       = var.db_instance_class

  allocated_storage     = var.db_allocated_storage
  max_allocated_storage = var.db_max_allocated_storage
  storage_encrypted     = true

  db_name  = var.db_name
  username = var.db_username
  password = random_password.db.result
  port     = 5432

  # Password is managed by Terraform + Secrets Manager above, not RDS-managed.
  manage_master_user_password = false

  multi_az               = var.db_multi_az
  vpc_security_group_ids = [aws_security_group.db.id]
  subnet_ids             = module.vpc.private_subnets
  create_db_subnet_group = true

  backup_retention_period = var.db_backup_retention_days
  deletion_protection     = var.db_deletion_protection
  skip_final_snapshot     = !var.db_deletion_protection

  performance_insights_enabled    = true
  enabled_cloudwatch_logs_exports = ["postgresql"]

  tags = local.tags
}
