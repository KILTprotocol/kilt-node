FROM amazon/aws-cli:2.7.31

RUN amazon-linux-extras install docker \
    && yum install -y jq
