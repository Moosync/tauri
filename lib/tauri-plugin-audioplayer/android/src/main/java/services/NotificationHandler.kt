package app.moosync.audioplayer.services

import android.app.Activity
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.support.v4.media.session.MediaSessionCompat
import android.util.Log
import androidx.core.app.NotificationCompat
import app.moosync.audioplayer.AudioPlayerPlugin
import app.moosync.audioplayer.R
import app.moosync.audioplayer.services.Constants.NOTIFICATION_CHANNEL_ID
import app.moosync.audioplayer.services.Constants.NOTIFICATION_ID

class NotificationHandler (
    private val mContext: Context,
    private val token: MediaSessionCompat.Token,
    private val launcherIcon: Int,
) {
    private val notificationManager: NotificationManager =
        mContext.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager

    private var notificationBuilder: NotificationCompat.Builder =
        NotificationCompat.Builder(mContext, NOTIFICATION_CHANNEL_ID)

    var notification: Notification? = null

    init {
        // Cancel all notifications
        notificationManager.cancelAll()
        createNotificationChannel()
    }

    private fun createNotificationChannel() {
        val existingChannel = notificationManager.getNotificationChannel(NOTIFICATION_CHANNEL_ID)
        if (existingChannel == null) {
            val channel = NotificationChannel(
                NOTIFICATION_CHANNEL_ID,
                "Now playing",
                NotificationManager.IMPORTANCE_LOW
            )
            channel.enableLights(false)
            channel.enableVibration(false)
            channel.setShowBadge(false)

            notificationManager.createNotificationChannel(channel)
        }
    }

    private fun createNotification() {
        val mediaStyle = androidx.media.app.NotificationCompat.MediaStyle()
            .setMediaSession(token).setShowActionsInCompactView(0, 1, 2)

        val clickIntent = PendingIntent
            .getActivity(
                mContext,
                0,
                Intent(mContext, AudioPlayerPlugin::class.java),
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
            )

        notificationBuilder
            .setStyle(mediaStyle)
            .setVisibility(NotificationCompat.VISIBILITY_PUBLIC)
            .setSmallIcon(launcherIcon)
            .setContentIntent(clickIntent)
            .setShowWhen(false)
            .build()

        notification = notificationBuilder.build()
    }

    fun clearNotification() {
        Log.d("TAG", "clearNotification: clearing")
        notificationManager.cancel(NOTIFICATION_ID)
    }

    fun updateMetadata() {
        if (notification == null) {
            createNotification()
        }
        notificationManager.notify(NOTIFICATION_ID, notification)
    }

    fun release() {
        notificationManager.cancelAll()
    }
}