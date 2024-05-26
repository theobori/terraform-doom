resource "local_sensitive_file" "test1" {
  content  = "Test sensitive content !"
  filename = "${var.base_path}/test1"
}

resource "local_file" "test2" {
  for_each = toset([ "1a", "2b", "3c" ])
  content  = "${each.value} Test content !\n"
  filename = "${var.base_path}/${each.value}"
}
