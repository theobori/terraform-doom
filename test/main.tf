resource "local_file" "test1" {
  content  = "Test content !"
  filename = "${var.base_path}/test1"
}

resource "local_file" "test2" {
  content  = "Test content !"
  filename = "${var.base_path}/test2"
}

resource "local_sensitive_file" "test3" {
  content  = "Test sensitive content !"
  filename = "${var.base_path}/test3"
}
