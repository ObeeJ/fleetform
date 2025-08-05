resource "aws_instance" "web" {
  ami           = var.ami_id
  instance_type = var.instance_type
  subnet_id     = var.subnet_id

  tags = {
    Name = var.instance_name
  }
}

variable "ami_id" {
  description = "AMI ID for EC2 instance"
  type        = string
  default     = "ami-0abcdef1234567890"
}

variable "instance_type" {
  description = "EC2 instance type"
  type        = string
  default     = "t3.micro"
}

variable "subnet_id" {
  description = "Subnet ID for EC2 instance"
  type        = string
}

variable "instance_name" {
  description = "Name of the EC2 instance"
  type        = string
  default     = "fleetform-web"
}

output "instance_id" {
  value = aws_instance.web.id
}