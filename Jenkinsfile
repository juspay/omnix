// We use https://github.com/juspay/jenkins-nix-ci

pipeline {
    agent any
    stages {
        stage ('Build') {
            steps {
                // https://github.com/srid/nixci
                nixCI ()
            }
        }
        /* stage ('Cachix push') {
            when { branch 'master' }
            steps {
                cachixPush "srid"
            }
        }
        */
    }
}
