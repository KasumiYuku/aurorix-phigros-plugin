# {{title}}

``` text
曲师      {{composer}}
画师      {{illustrator}}
章节      {{chapter}}
BPM       {{bpm}}
时长      {{length}}
```

{{#each plays}}
**{{difficulty}}** {{constant}}
``` text
分数      {{score}}    评级 {{grade}}
ACC       {{acc}}
单曲 RKS  {{rks}}    排名 {{rank}}
{{suggest}}
```
{{/each}}

{{footer}}
