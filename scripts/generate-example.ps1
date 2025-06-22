# Author: MarrieMitsu
# Usage: ./genereate-example -Amount 1000 -SeedPath "target/path/seed.json" -OutputDir "target/path/"

param(
    [Parameter(Mandatory=$true)]
    [int]$Amount,

    [Parameter(Mandatory=$true)]
    [string]$SeedPath,

    [Parameter(Mandatory=$true)]
    [string]$OutputDir
)

function Get-RandomDateTime {
    $start = Get-Date "01.01.2020"
    $end = Get-Date "12.31.2025"

    return Get-Random -Minimum $start.Ticks -Maximum $end.Ticks
}


$seed = Get-Content -Path $SeedPath | ConvertFrom-Json
$v1 = @();

for ($i = 0; $i -lt $Amount; $i++) {
    $item = Get-Random -InputObject $seed
    $datetime = ([datetime](Get-RandomDateTime)).toUniversalTime().ToString("yyyy-MM-ddTHH:mm:ss.fffZ")

    $v1Entry = [PSCustomObject]@{
        header = "YouTube"
        title = "Watched $($item.title)"
        titleUrl = "https://www.youtube.com/watch?v=$($item.id)"
        subtitles = @([PSCustomObject]@{
            name = $item.channel_name
            url = "https://www.youtube.com/channel/$($item.channel_id)"
        })
        time = $dateTime
        products = @("YouTube")
        activityControls = @("YouTube watch history")
    }

    $v1 += $v1Entry
}

$v1 | ConvertTo-Json -Depth 4 -Compress | Set-Content "$($OutputDir)/v1-watch-history.json"

Write-Output "Generated v1-watch-history.json with $Amount entries."
