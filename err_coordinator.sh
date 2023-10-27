tmux new-session -d -s backfill-errors-session

for i in {1..20}; do
    tmux new-window -t backfill-errors-session: -n "back-$i"
    tmux send-keys -t backfill-errors-session:"back-$i" "python ./backfill_errors.py $i" C-m
done

tmux attach-session -t backfill-errors-session