# GKE cluster hosting the LawSynth services (api, scheduler, worker, artifact).

resource "google_container_cluster" "cluster" {
  name     = "${local.name}-gke"
  location = var.region
  network  = google_compute_network.vpc.id

  subnetwork = google_compute_subnetwork.nodes.id

  # Manage node pools separately: create the cluster with a removable default.
  remove_default_node_pool = true
  initial_node_count       = 1

  release_channel {
    channel = var.release_channel
  }

  min_master_version = var.kubernetes_version != "" ? var.kubernetes_version : null

  # VPC-native (alias IP) using the secondary ranges defined in network.tf.
  networking_mode = "VPC_NATIVE"
  ip_allocation_policy {
    cluster_secondary_range_name  = "pods"
    services_secondary_range_name = "services"
  }

  workload_identity_config {
    workload_pool = "${var.project_id}.svc.id.goog"
  }

  # Guard against accidental deletion of a running research cluster.
  deletion_protection = true

  depends_on = [google_project_service.services]
}

resource "google_container_node_pool" "workers" {
  name     = "workers"
  location = var.region
  cluster  = google_container_cluster.cluster.name

  initial_node_count = var.worker_initial_nodes

  autoscaling {
    min_node_count = var.worker_min_nodes
    max_node_count = var.worker_max_nodes
  }

  management {
    auto_repair  = true
    auto_upgrade = true
  }

  node_config {
    machine_type = var.worker_machine_type
    disk_size_gb = 100
    disk_type    = "pd-ssd"

    oauth_scopes = ["https://www.googleapis.com/auth/cloud-platform"]

    workload_metadata_config {
      mode = "GKE_METADATA"
    }

    labels = {
      "lawsynth_io_pool" = "workers"
    }

    shielded_instance_config {
      enable_secure_boot          = true
      enable_integrity_monitoring = true
    }
  }
}
