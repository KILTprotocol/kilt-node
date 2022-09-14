FROM amazon/aws-cli:2.7.31
RUN yum install -y curl \
  && yum install -y jq

RUN amazon-linux-extras install docker
