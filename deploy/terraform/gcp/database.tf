# Cloud SQL for PostgreSQL backing LawSynth metadata: run specs, artifact
# references, and append-only run events.

resource "random_password" "db" {
  length  = 32
  special = false
}

resource "random_id" "db_suffix" {
  byte_length = 2
}

resource "google_sql_database_instance" "postgres" {
  # Instance names cannot be reused for ~1 week after deletion; a suffix avoids
  # collisions across recreate cycles.
  name             = "${local.name}-pg-${random_id.db_suffix.hex}"
  region           = var.region
  database_version = var.postgres_version

  deletion_protection = var.db_deletion_protection

  depends_on = [google_service_networking_connection.private_service]

  settings {
    tier              = var.db_tier
    availability_type = var.db_availability_type
    disk_size         = var.db_disk_size
    disk_type         = "PD_SSD"
    disk_autoresize   = true

    backup_configuration {
      enabled                        = true
      point_in_time_recovery_enabled = true
      backup_retention_settings {
        retained_backups = var.db_backup_retention
      }
    }

    ip_configuration {
      # Private IP only; reachable from the GKE nodes over the peered network.
      ipv4_enabled    = false
      private_network = google_compute_network.vpc.id
    }

    insights_config {
      query_insights_enabled = true
    }

    user_labels = local.labels
  }
}

resource "google_sql_database" "lawsynth" {
  name     = var.db_name
  instance = google_sql_database_instance.postgres.name
}

resource "google_sql_user" "lawsynth" {
  name     = var.db_username
  instance = google_sql_database_instance.postgres.name
  password = random_password.db.result
}
