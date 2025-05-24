# BoostzzkNfCA9D1qNuN5xZxB5ErbK4zQuBeTHGDpXT1 - Boost program
# Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB - Meteora pools program
# 24Uqj9JCLxUeoC3hGfh5W3s9FM9uCHDS2SG3LYwBpyTi - Meteora vault program
# 2G3WKgP9Eemzgbzk5B4w2JSAizjWfEmEwa4JMfzLyXzM - ORE config
# Dh5ZkjGD8EVujR7C8mxMyYaE2LRVarJ9W6bMofTgNJFP - Treasury
# HqPcY2CUB4FL5EAGWN1yZkS6DHYUoMsnjoSpdGqV8wPC - Treasury tokens
# oreoU2P8bN6jkk3jbaiVxYnG1dCXcYxwhwyK9jSybcp - ORE mint
# DvYP7L1dH6vK4CbEKtQehP9cSZC5qXFP53NQi5THt5U5 - Boost proof
# FQpx4mmybtZSwv8G5QwfyXWYjRt9nkqiw3sAsKGiUpCG - Boost config
# GgaDTFbqdgjoZz3FP7zrtofGwnRS4E6MCzmmD5Ni1Mxj - ORE-SOL pool
# 3s6ki6dQSM8FuqWiPsnGkgVsAEo8BTAfUR1Vvt1TPiJN - A vault
# FERjPVNEa7Udq8CEv68h6tPL46Tq7ieE49HrE2wea3XT - B vault
# BtJuiRG44vew5nYBVeUhuBawPTZLyYYxdzTYzerkfnto - A token vault
# HZeLxbZ9uHtSpwZC3LBr4Nubd14iHwz7bRSghRZf5VCG - B token vault
# 6Av9sdKvnjwoDHVnhEiz6JEq8e6SGzmhCsCncT2WJ7nN - A vault LP mint
# FZN7QZ8ZUUAxMPfxYEYkH3cXUASzH8EqA6B4tyCL8f1j - B vault LP mint
# 2k7V1NtM1krwh1sdt5wWqBRcvNQ5jzxj3J2rV78zdTsL - A vault LP
# CFATQFgkKXJyU3MdCNvQqN79qorNSMJFF8jrF66a7r6i - B vault LP
# 3WYz5TC8X4FLvwWQ2QvSfXuZHXjqvsdymKwmMFkgCgVs - Protocol token fee A
# 6kzYo2LMo2q2bkLAD8ienoG5NC1MkNXNTfm8sdyHuX3h - Protocol token fee B
solana-test-validator \
    -r \
    --bpf-program oreV2ZymfyeXgNgBdqMkumTqqAprVqgBWQfoYkrtKWQ target/deploy/ore.so \
    --url https://api.mainnet-beta.solana.com \
    --clone-upgradeable-program BoostzzkNfCA9D1qNuN5xZxB5ErbK4zQuBeTHGDpXT1 \
    --clone-upgradeable-program Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB \
    --clone-upgradeable-program 24Uqj9JCLxUeoC3hGfh5W3s9FM9uCHDS2SG3LYwBpyTi \
    --clone 2G3WKgP9Eemzgbzk5B4w2JSAizjWfEmEwa4JMfzLyXzM \
    --clone Dh5ZkjGD8EVujR7C8mxMyYaE2LRVarJ9W6bMofTgNJFP \
    --clone HqPcY2CUB4FL5EAGWN1yZkS6DHYUoMsnjoSpdGqV8wPC \
    --clone oreoU2P8bN6jkk3jbaiVxYnG1dCXcYxwhwyK9jSybcp \
    --clone DvYP7L1dH6vK4CbEKtQehP9cSZC5qXFP53NQi5THt5U5 \
    --clone FQpx4mmybtZSwv8G5QwfyXWYjRt9nkqiw3sAsKGiUpCG \
    --clone GgaDTFbqdgjoZz3FP7zrtofGwnRS4E6MCzmmD5Ni1Mxj \
    --clone 3s6ki6dQSM8FuqWiPsnGkgVsAEo8BTAfUR1Vvt1TPiJN \
    --clone FERjPVNEa7Udq8CEv68h6tPL46Tq7ieE49HrE2wea3XT \
    --clone BtJuiRG44vew5nYBVeUhuBawPTZLyYYxdzTYzerkfnto \
    --clone HZeLxbZ9uHtSpwZC3LBr4Nubd14iHwz7bRSghRZf5VCG \
    --clone 6Av9sdKvnjwoDHVnhEiz6JEq8e6SGzmhCsCncT2WJ7nN \
    --clone FZN7QZ8ZUUAxMPfxYEYkH3cXUASzH8EqA6B4tyCL8f1j \
    --clone 2k7V1NtM1krwh1sdt5wWqBRcvNQ5jzxj3J2rV78zdTsL \
    --clone CFATQFgkKXJyU3MdCNvQqN79qorNSMJFF8jrF66a7r6i \
    --clone 3WYz5TC8X4FLvwWQ2QvSfXuZHXjqvsdymKwmMFkgCgVs \
    --clone 6kzYo2LMo2q2bkLAD8ienoG5NC1MkNXNTfm8sdyHuX3h
