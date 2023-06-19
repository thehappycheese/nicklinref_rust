# Set your paths and files
$distFolder = "./dist"
$gitIgnoreFile = "./dist/.gitignore"
$zipFile = ""
$itemsToZip = @("./__static_http", "./target/release/nicklinref.exe", "README.md", "LICENSE")

# Create dist folder if it doesn't exist
If(!(Test-Path -Path $distFolder )){
    New-Item -ItemType directory -Path $distFolder
}

# Create or overwrite .gitignore
Set-Content -Path $gitIgnoreFile -Value "*"

# Read version from cargo.toml
$cargoToml = Get-Content -Path "./Cargo.toml"
$versionLine = $cargoToml | Select-String -Pattern 'version\s*=\s*"([^"]*)"'
if($versionLine -ne $null){
    $version = $versionLine.Matches.Groups[1].Value
    $zipFile = "./dist/$version.zip"
}else{
    Write-Host "Couldn't find version from Cargo.toml"
    exit
}

# Create the zip file with the items
If(Test-Path -Path $zipFile){
    Remove-Item -Path $zipFile
}

Compress-Archive -Path $itemsToZip -DestinationPath $zipFile

Write-Host "Distribution package is ready."