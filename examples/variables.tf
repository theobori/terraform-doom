variable "base_path" {
  description = "Base path for the local files"
  type        = string
  default     = "/tmp"
}

variable "files_amount" {
  description = "Amount of created local files"
  type        = number
  default     = 5
}
