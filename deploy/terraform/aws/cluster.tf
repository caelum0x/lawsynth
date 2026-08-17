# EKS cluster hosting the LawSynth services (api, scheduler, worker, artifact).

module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 20.8"

  cluster_name    = "${local.name}-eks"
  cluster_version = var.kubernetes_version

  cluster_endpoint_public_access       = true
  cluster_endpoint_public_access_cidrs = var.cluster_public_access_cidrs

  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnets

  # Grant the caller cluster-admin so kubectl/helm work immediately post-apply.
  enable_cluster_creator_admin_permissions = true

  cluster_addons = {
    coredns    = {}
    kube-proxy = {}
    vpc-cni    = {}
  }

  eks_managed_node_group_defaults = {
    ami_type = "AL2023_x86_64_STANDARD"
  }

  eks_managed_node_groups = {
    workers = {
      instance_types = var.worker_instance_types
      min_size       = var.worker_min_size
      max_size       = var.worker_max_size
      desired_size   = var.worker_desired_size

      labels = {
        "lawsynth.io/pool" = "workers"
      }
    }
  }

  tags = local.tags
}
