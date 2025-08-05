resource "aws_instance" "example" {
  ami           = "ami-12345678"
  instance_type = "t2.micro"
}

resource "aws_s3_bucket" "my_bucket" {
  bucket = "my-bucket"
}
