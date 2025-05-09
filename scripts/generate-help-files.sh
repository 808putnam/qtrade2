#!/bin/bash

# Notes:
# 1. Solana cli section is deprecated.
# Created before I found this:
# https://docs.solanalabs.com/cli/usage
# Leaving in for now, but will likely remove in the future.
# 2. Other sections still have value.

# Display help menu
usage() {
    echo ""
    echo "generate-help-files.sh"
    echo "========================"
    echo ""
    echo "Loop over a variety of tooling and"
    echo "generate help files for each tool."
    echo ""
    echo "--help   Print help for script"
    echo ""

    exit
}

# Parse input arguments
for i in "$@"
do
case $i in
    -h|--help)
    usage
    shift
    ;;
    *)
    echo "Unknown option: $i"
    usage
    shift
    ;;
esac
done

# Clean up any previous help files
if [ -d help-files ]; then rm -rf help-files; fi

# Stage for --help calls
mkdir -p help-files
cd help-files

# Centralize our --help calls for root commands (e.g., solana --help)
root_command_help_file() {

    # Grab parameter
    local ROOT_COMMAND=$1

    # Create the help file, overwriting any previous file
    $ROOT_COMMAND --help > ${ROOT_COMMAND}-help.txt
}

sub_command_help_file() {

    # Grab parameters
    local ROOT_COMMAND=$1
    local SUB_COMMAND=$2

    # Create the help file, overwriting any previous file
    $ROOT_COMMAND $SUB_COMMAND --help > ${ROOT_COMMAND}-${SUB_COMMAND}-help.txt
}

# Build out anchor cli help files
root_command_help_file anchor
sub_command_help_file anchor init    
sub_command_help_file anchor build   
sub_command_help_file anchor expand  
sub_command_help_file anchor verify  
sub_command_help_file anchor test    
sub_command_help_file anchor new     
sub_command_help_file anchor idl     
sub_command_help_file anchor clean   
sub_command_help_file anchor deploy  
sub_command_help_file anchor migrate 
sub_command_help_file anchor upgrade 
sub_command_help_file anchor cluster 
sub_command_help_file anchor shell   
sub_command_help_file anchor run     
sub_command_help_file anchor login   
sub_command_help_file anchor publish 
sub_command_help_file anchor keys    
sub_command_help_file anchor localnet
sub_command_help_file anchor account 

# Build out solana cli help files
root_command_help_file solana
sub_command_help_file solana account
sub_command_help_file solana address                          
sub_command_help_file solana address-lookup-table             
sub_command_help_file solana airdrop                          
sub_command_help_file solana authorize-nonce-account

sub_command_help_file solana balance                          
sub_command_help_file solana block                            
sub_command_help_file solana block-height                     
sub_command_help_file solana block-production                 
sub_command_help_file solana block-time

sub_command_help_file solana catchup                          
sub_command_help_file solana close-vote-account               
sub_command_help_file solana cluster-date                                                
sub_command_help_file solana cluster-version                  
sub_command_help_file solana completion                       
sub_command_help_file solana config                           
sub_command_help_file solana confirm                          
sub_command_help_file solana create-address-with-seed                                  
sub_command_help_file solana create-nonce-account             
sub_command_help_file solana create-stake-account             
sub_command_help_file solana create-stake-account-checked     
sub_command_help_file solana create-vote-account

sub_command_help_file solana deactivate-stake                 
sub_command_help_file solana decode-transaction               
sub_command_help_file solana delegate-stake

sub_command_help_file solana epoch                            
sub_command_help_file solana epoch-info 

sub_command_help_file solana feature                          
sub_command_help_file solana fees                             
sub_command_help_file solana find-program-derived-address     
sub_command_help_file solana first-available-block 

sub_command_help_file solana genesis-hash                     
sub_command_help_file solana gossip   

sub_command_help_file solana inflation      

sub_command_help_file solana largest-accounts                 
sub_command_help_file solana leader-schedule                  
sub_command_help_file solana live-slots                       
sub_command_help_file solana logs           

sub_command_help_file solana merge-stake  

sub_command_help_file solana new-nonce                        
sub_command_help_file solana nonce                            
sub_command_help_file solana nonce-account      

sub_command_help_file solana ping                             
sub_command_help_file solana program                          
sub_command_help_file solana program-v4         

sub_command_help_file solana redelegate-stake                 
sub_command_help_file solana rent                             
sub_command_help_file solana resolve-signer           

sub_command_help_file solana sign-offchain-message            
sub_command_help_file solana slot                             
sub_command_help_file solana split-stake                      
sub_command_help_file solana stake-account                    
sub_command_help_file solana stake-authorize                  
sub_command_help_file solana stake-authorize-checked                                
sub_command_help_file solana stake-history                    
sub_command_help_file solana stake-minimum-delegation         
sub_command_help_file solana stake-set-lockup                 
sub_command_help_file solana stake-set-lockup-checked         
sub_command_help_file solana stakes                           
sub_command_help_file solana supply                     

sub_command_help_file solana transaction-count                
sub_command_help_file solana transaction-history                                          
sub_command_help_file solana transfer                   

sub_command_help_file solana upgrade-nonce-account       

sub_command_help_file solana validator-info                   
sub_command_help_file solana validators                       
sub_command_help_file solana verify-offchain-signature        
sub_command_help_file solana vote-account                     
sub_command_help_file solana vote-authorize-voter             
sub_command_help_file solana vote-authorize-voter-checked                                  
sub_command_help_file solana vote-authorize-withdrawer        
sub_command_help_file solana vote-authorize-withdrawer-checked                               
sub_command_help_file solana vote-update-commission           
sub_command_help_file solana vote-update-validator         
   
sub_command_help_file solana wait-for-max-stake               
sub_command_help_file solana withdraw-from-nonce-account      
sub_command_help_file solana withdraw-from-vote-account       
sub_command_help_file solana withdraw-stake                   

# Build out solana-keygen cli help files
root_command_help_file solana-keygen
sub_command_help_file solana-keygen grind  
sub_command_help_file solana-keygen new    
sub_command_help_file solana-keygen pubkey 
sub_command_help_file solana-keygen recover
sub_command_help_file solana-keygen verify 

# Build out solana-test-validator cli help files
root_command_help_file solana-test-validator

# Build out spl-token cli help files
root_command_help_file spl-token
sub_command_help_file spl-token accounts                             
sub_command_help_file spl-token address                              
sub_command_help_file spl-token apply-pending-balance                
sub_command_help_file spl-token approve                              
sub_command_help_file spl-token authorize

sub_command_help_file spl-token balance                              
sub_command_help_file spl-token bench                                
sub_command_help_file spl-token burn

sub_command_help_file spl-token close                                
sub_command_help_file spl-token close-mint                           
sub_command_help_file spl-token configure-confidential-transfer-account
sub_command_help_file spl-token create-account                       
sub_command_help_file spl-token create-multisig                      
sub_command_help_file spl-token create-token

sub_command_help_file spl-token deposit-confidential-tokens          
sub_command_help_file spl-token disable-confidential-credits         
sub_command_help_file spl-token disable-cpi-guard                    
sub_command_help_file spl-token disable-non-confidential-credits     
sub_command_help_file spl-token disable-required-transfer-memos      
sub_command_help_file spl-token display

sub_command_help_file spl-token enable-confidential-credits
sub_command_help_file spl-token enable-cpi-guard                     
sub_command_help_file spl-token enable-non-confidential-credits      
sub_command_help_file spl-token enable-required-transfer-memos

sub_command_help_file spl-token freeze

sub_command_help_file spl-token gc

sub_command_help_file spl-token initialize-metadata

sub_command_help_file spl-token mint

sub_command_help_file spl-token revoke 

sub_command_help_file spl-token set-interest-rate                    
sub_command_help_file spl-token set-transfer-fee                     
sub_command_help_file spl-token set-transfer-hook                    
sub_command_help_file spl-token supply                               
sub_command_help_file spl-token sync-native

sub_command_help_file spl-token thaw                                 
sub_command_help_file spl-token transfer
                             
sub_command_help_file spl-token unwrap                               
sub_command_help_file spl-token update-confidential-transfer-settings
sub_command_help_file spl-token update-default-account-state
