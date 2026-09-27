resource "aws_instance" "example" {
  # Placeholder AMI is resolved at apply time to current Amazon Linux 2023
  # unless FLEETFORM_AMI is set.
  ami           = "ami-12345678"
  instance_type = "t3.micro"
}
