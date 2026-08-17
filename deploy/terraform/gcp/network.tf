# VPC-native network for GKE plus Cloud NAT for private node egress.

resource "google_compute_network" "vpc" {
  name                    = "${local.name}-vpc"
  auto_create_subnetworks = false
  depends_on              = [google_project_service.services]
}

resource "google_compute_subnetwork" "nodes" {
  name          = "${local.name}-nodes"
  region        = var.region
  network       = google_compute_network.vpc.id
  ip_cidr_range = var.subnet_cidr

  private_ip_google_access = true

  secondary_ip_range {
    range_name    = "pods"
    ip_cidr_range = var.pods_cidr
  }

  secondary_ip_range {
    range_name    = "services"
    ip_cidr_range = var.services_cidr
  }
}

resource "google_compute_router" "router" {
  name    = "${local.name}-router"
  region  = var.region
  network = google_compute_network.vpc.id
}

resource "google_compute_router_nat" "nat" {
  name                               = "${local.name}-nat"
  router                             = google_compute_router.router.name
  region                             = var.region
  nat_ip_allocate_option             = "AUTO_ONLY"
  source_subnetwork_ip_ranges_to_nat = "ALL_SUBNETWORKS_ALL_IP_RANGES"

  log_config {
    enable = true
    filter = "ERRORS_ONLY"
  }
}

# Private Services Access so Cloud SQL is reachable over a private IP.
resource "google_compute_global_address" "private_service_range" {
  name          = "${local.name}-psa"
  purpose       = "VPC_PEERING"
  address_type  = "INTERNAL"
  prefix_length = 16
  network       = google_compute_network.vpc.id
}

resource "google_service_networking_connection" "private_service" {
  network                 = google_compute_network.vpc.id
  service                 = "servicenetworking.googleapis.com"
  reserved_peering_ranges = [google_compute_global_address.private_service_range.name]
}
