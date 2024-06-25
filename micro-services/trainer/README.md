# Sudoku ML Trainer

This micro-service (see [my rant](#rant)) will most likely be for dev purpose and will not be running in production.  Perhaps for devops, they would love to have some kind of ML Trainer running in realtime on production so they can do segmentations based on characteristics, but for this purpose, it's just a service in which the input for ML training is collected on the client-side, and passed/sent over to this trainer.

Technologically speaking, I'm hoping/thinking that the trained result will be saved/persisted/preserved as [ONNX](https://github.com/onnx/onnx) format, mainly because (IMHO) I think it's the most universal.  And under the hood, they use Protobuf for the tools, which I am biased.  I am also biased towards chosing libraries which supports Microsoft (i.e. ONNX runtime on C# for Azure, etc.) based on all of my past experience in software development (oh, with one exception: Docker for Windows, it's the worst hack, and I hate it, even Docker+WSL2, it's horrible!  Kubernetes on Windows? Don't get me started!!!! :disappointed: :tired_face:).  Why bias towards Microsoft?  Very simple reason, how many libraries do you know that are not backed by Microsoft that existed for lifetime of your project?  Your project/software probably became deprecated because that library is no longer supported...  By the way, [libVLC](https://github.com/videolan/vlc) and [FFmpeg](https://github.com/FFmpeg/FFmpeg) are the only two libraries that currently pops to my head at the moment, but the point is, not much libraries survives for a very long term.

## API

As mented everywhere, I am more of a data-contract kind of programmer, where I like to mold things based on the shape of the data-model...  The gRPC based Protobuf model for training is as follows:

```protobuf
syntax = "proto3";
```

## Rant

You can skip this section for all I am doing is ranting about usage of ML more than anything technical...

This is a special micro-service, in which in production server, it will highly unlikely be running.  One may argue that because the routing/proxy service (i.e. Nginx, Apache, IIS, Google Load-Balancer, etc) is the only WAN exposing endpoint, it's benign to have it running.  This is true, but financially, it cost money to have each VM running on the cloud, so it's more about cutting cost on unnecessary services when possible.  The image can/may reside on the cloud as long as it's not running.  In most cases, cloud services tends to suck up :yen:円 :dollar:$ usage cost :moneybag: on storage (I once got charged by Microsoft Azure for close to 12 months for storage fees when I had deleted the VM but had forgotten to get rid of the storage ::crying_cat_face:).  Incidentally, I'm still getting charged few cents (about $0.12/months) monthly from Amazon AWS even though I've completely deleted/closed my account from AWS, and it's getting charged to our Amazon.com (shopping) account.  Can anybody tell me how to deal with this (since the account is closed, I've no way to look at dashboard or anything).  I've also got ripped off from Google Cloud as well in which I was getting charged for 3 Google Voice numbers (licenses) when I only have 1 VOIP number, so I'm not too lucky with cloud services when it comes to being charged for usages on services I don't need :worried:.  I stopped using Google Colab in fear that Google will charge me for something accidentally triggered, so I now use Kaggle in which it is not tied to my Google Pay, Amazon shopping, or Microsoft Store!

As long as we're on the topic of financial issues associated to ML, make sure that any cloud services you'd get charged are as much as possible, pay-as-you-use model rather than monthly based.  For example, if you have Github Copilot (like I do), you'd realize after a while that you only use it once in a while, and you start to wonder if you were getting charged by usage rather than monthly ($10.00/month), you'd be charged less than $10.00.  Sure, when you really need it, it's very much useful (I'm unsure still whether it's worth it, but I will say it's useful) because at times, I still have to switch over to ChatGPT for technical questions that are not programming related (i.e. I'd ask "what are the emoji for '😿' in Github README.md markdown?", and I'd get answers that basically says that it's not programming question, so it cannot help me; and because I am using [vim-plugin](https://github.com/github/copilot.vim) on console, I don't have access to GUI!).

It's very useful (though it hogs a lot of RAM) when you're not on GUI, but is it worth $10/month?  In any case, the point is, I really like the idea of per-query, per-minute, limit-to-X-queries-per-minute type of micro-transaction charging based over monthly/fixed prize usage, because most of the time, you don't use/need ML services...

But then again, there are many YouTube videos of which the author would claim that these LLM based AI wrote flawless code, but I challenge them to redo that not in Python or Java, but either in C++, Rust, or even C#.  For the record, both Python and Java are popular mostly because of the vast collections of libraries (prove me wrong!).  In fact, during the period when portability was quite needed, Microsoft was incredibly stubborn about .NET libraries (in specific, WCF and ASP.NET) to be ported to MONO, and about that time, Python began becoming more popular (though some may argue that Python math library is what boosted popularity amongst ML scientists at Google (Tensor?), etc, etc).  All in all, it points to libraries more than language itself...  So the sample code are more vastly available in the source-code jungle and just happens to not be MIT, Apache, CC, or no-licensed (or copyleft?) so it was allowed to for LLM's to consume it.  Again, I challenge these people who claims "AI coded perfectly" to have it code in languages that libraries are not yet stable, or even in Python and/or Java where libraries do not exist (or it exists, but the library emerged about the time of late 2023 when ChatGPT3 was out, so the LLM doesn't know that these new libraries exist), to write logics.  I did not mean to rant, but these people who are using LLM based queries to "write code" do not impress me at all.
